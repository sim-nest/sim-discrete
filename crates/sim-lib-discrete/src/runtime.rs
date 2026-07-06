//! The `discrete:` runtime lib: live kernel registration of the discrete-math
//! family as callable operations.
//!
//! Each op closes over no state and is registered as a `core/Function`. To stay
//! decoupled from any number domain (the cookbook precedent), arguments and
//! results are strings: integers are decimal text, and vectors/matrices use the
//! read-construct forms from [`crate::forms`]. This binds the discrete crates to
//! a live `Cx` without pulling a numeric codec into the kernel boundary.

use std::sync::Arc;

use sim_kernel::{
    AbiVersion, Args, Callable, ClassRef, Cx, Error, Export, Expr, Lib, LibManifest, LibTarget,
    Linker, LoadCx, Object, ObjectCompat, Result, Symbol, Value, Version,
};

use crate::comb;
use crate::forms;
use crate::graph;
use crate::spectral;
#[cfg(feature = "citizen")]
use crate::{FwhtSignalDescriptor, MatrixDescriptor};

/// Which discrete operation a [`DiscreteOp`] performs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpKind {
    /// `discrete:binomial "n" "k"` -> decimal string.
    Binomial,
    /// `discrete:factorial "n"` -> decimal string.
    Factorial,
    /// `discrete:partition-count "n"` -> decimal string.
    PartitionCount,
    /// `discrete:fwht "<fwht-signal form>"` -> fwht-signal form string.
    Fwht,
    /// `discrete:xor-convolve "<form a>" "<form b>"` -> fwht-signal form string.
    XorConvolve,
    /// `discrete:mst-weight "<matrix form>"` -> decimal MST weight string.
    MstWeight,
}

impl OpKind {
    /// Every op kind, for registration.
    pub const ALL: [OpKind; 6] = [
        OpKind::Binomial,
        OpKind::Factorial,
        OpKind::PartitionCount,
        OpKind::Fwht,
        OpKind::XorConvolve,
        OpKind::MstWeight,
    ];

    fn name(self) -> &'static str {
        match self {
            OpKind::Binomial => "binomial",
            OpKind::Factorial => "factorial",
            OpKind::PartitionCount => "partition-count",
            OpKind::Fwht => "fwht",
            OpKind::XorConvolve => "xor-convolve",
            OpKind::MstWeight => "mst-weight",
        }
    }

    /// The `discrete:<name>` symbol this op registers under.
    pub fn symbol(self) -> Symbol {
        Symbol::qualified("discrete", self.name())
    }

    fn arity(self) -> usize {
        match self {
            OpKind::XorConvolve => 2,
            OpKind::Binomial => 2,
            _ => 1,
        }
    }
}

/// A registered discrete operation.
pub struct DiscreteOp {
    kind: OpKind,
}

impl DiscreteOp {
    /// Build the op for `kind`.
    pub fn new(kind: OpKind) -> Self {
        Self { kind }
    }
}

fn eval_err(msg: impl Into<String>) -> Error {
    Error::Eval(msg.into())
}

fn string_arg(cx: &mut Cx, value: &Value) -> Result<String> {
    #[cfg(feature = "citizen")]
    if let Some(form) = citizen_form_arg(value) {
        return Ok(form);
    }
    match value.object().as_expr(cx)? {
        Expr::String(text) => Ok(text),
        _ => Err(eval_err("discrete op expects a string argument")),
    }
}

#[cfg(feature = "citizen")]
fn citizen_form_arg(value: &Value) -> Option<String> {
    if let Some(signal) = value.object().downcast_ref::<FwhtSignalDescriptor>() {
        return Some(signal.as_text().to_owned());
    }
    if let Some(matrix) = value.object().downcast_ref::<MatrixDescriptor>() {
        return Some(matrix.as_text().to_owned());
    }
    None
}

fn parse_u64(text: &str) -> Result<u64> {
    text.trim()
        .parse::<u64>()
        .map_err(|_| eval_err(format!("expected a non-negative integer, got {text:?}")))
}

impl DiscreteOp {
    fn run(&self, cx: &mut Cx, args: Vec<Value>) -> Result<Value> {
        if args.len() != self.kind.arity() {
            return Err(eval_err(format!(
                "{} expects {} argument(s), got {}",
                self.kind.symbol(),
                self.kind.arity(),
                args.len()
            )));
        }
        let result = match self.kind {
            OpKind::Binomial => {
                let n = parse_u64(&string_arg(cx, &args[0])?)?;
                let k = parse_u64(&string_arg(cx, &args[1])?)?;
                comb::binomial_checked(n, k, comb::MAX_BINOMIAL_INPUT)
                    .map_err(|e| eval_err(e.to_string()))?
                    .to_string()
            }
            OpKind::Factorial => {
                let n = parse_u64(&string_arg(cx, &args[0])?)?;
                comb::factorial_checked(n)
                    .map_err(|e| eval_err(e.to_string()))?
                    .to_string()
            }
            OpKind::PartitionCount => {
                let n = parse_u64(&string_arg(cx, &args[0])?)?;
                comb::integer_partition_count_checked(n)
                    .map_err(|e| eval_err(e.to_string()))?
                    .to_string()
            }
            OpKind::Fwht => {
                let form = string_arg(cx, &args[0])?;
                let coeffs =
                    forms::decode_fwht_signal(&form).map_err(|e| eval_err(e.to_string()))?;
                let out = spectral::fwht_i64(&coeffs).map_err(|e| eval_err(e.to_string()))?;
                forms::encode_fwht_signal(&out.values)
            }
            OpKind::XorConvolve => {
                let a = forms::decode_fwht_signal(&string_arg(cx, &args[0])?)
                    .map_err(|e| eval_err(e.to_string()))?;
                let b = forms::decode_fwht_signal(&string_arg(cx, &args[1])?)
                    .map_err(|e| eval_err(e.to_string()))?;
                let out =
                    spectral::xor_convolution_i64(&a, &b).map_err(|e| eval_err(e.to_string()))?;
                forms::encode_fwht_signal(&out)
            }
            OpKind::MstWeight => {
                let form = string_arg(cx, &args[0])?;
                mst_weight_from_matrix_form(&form)?
            }
        };
        cx.factory().string(result)
    }
}

/// Decode a `#(discrete/matrix v1 int n n [..])` symmetric weight adjacency
/// (0 = no edge) and return its MST total weight as a decimal string.
fn mst_weight_from_matrix_form(form: &str) -> Result<String> {
    let (rows, cols, data) = forms::decode_matrix(form).map_err(|e| eval_err(e.to_string()))?;
    if rows != cols {
        return Err(eval_err("mst-weight requires a square adjacency matrix"));
    }
    let n = rows;
    let mut g: graph::Graph<usize, u64> =
        graph::Graph::with_nodes((0..n).collect(), graph::Directedness::Undirected);
    for i in 0..n {
        for j in (i + 1)..n {
            let w = data[i * n + j];
            if w > 0 {
                g.add_edge(i, j, w as u64)
                    .map_err(|e| eval_err(e.to_string()))?;
            }
        }
    }
    let tree = graph::kruskals_mst(&g).map_err(|e| eval_err(e.to_string()))?;
    Ok(tree.total_weight.to_string())
}

impl Object for DiscreteOp {
    fn display(&self, _cx: &mut Cx) -> Result<String> {
        Ok(format!("#<function {}>", self.kind.symbol()))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ObjectCompat for DiscreteOp {
    fn class(&self, cx: &mut Cx) -> Result<ClassRef> {
        cx.resolve_class(&Symbol::qualified("core", "Function"))
    }

    fn as_callable(&self) -> Option<&dyn Callable> {
        Some(self)
    }
}

impl Callable for DiscreteOp {
    fn call(&self, cx: &mut Cx, args: Args) -> Result<Value> {
        self.run(cx, args.into_vec())
    }
}

/// The `sim:discrete` lib id.
pub fn manifest_name() -> Symbol {
    Symbol::qualified("sim", "discrete")
}

/// The discrete runtime lib: registers the `discrete:*` ops.
pub struct DiscreteLib;

impl Lib for DiscreteLib {
    fn manifest(&self) -> LibManifest {
        LibManifest {
            id: manifest_name(),
            version: Version(env!("CARGO_PKG_VERSION").to_owned()),
            abi: AbiVersion { major: 0, minor: 1 },
            target: LibTarget::HostRegistered,
            requires: Vec::new(),
            capabilities: Vec::new(),
            exports: op_exports(),
        }
    }

    fn load(&self, cx: &mut LoadCx, linker: &mut Linker<'_>) -> Result<()> {
        for kind in OpKind::ALL {
            let op = DiscreteOp::new(kind);
            linker.function_value(kind.symbol(), cx.factory().opaque(Arc::new(op))?)?;
        }
        Ok(())
    }
}

/// The `discrete:*` function exports.
pub fn op_exports() -> Vec<Export> {
    OpKind::ALL
        .into_iter()
        .map(|kind| Export::Function {
            symbol: kind.symbol(),
            function_id: None,
        })
        .collect()
}

/// Install the discrete runtime lib (idempotent).
pub fn install_discrete_lib(cx: &mut Cx) -> Result<()> {
    sim_lib_core::install_once(cx, &DiscreteLib)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_test_support::core_cx;
    #[cfg(feature = "citizen")]
    use std::sync::Arc;

    fn call(cx: &mut Cx, kind: OpKind, args: Vec<&str>) -> Result<String> {
        let values = args
            .into_iter()
            .map(|a| cx.factory().string(a.to_string()))
            .collect::<Result<Vec<_>>>()?;
        call_values(cx, kind, values)
    }

    fn call_values(cx: &mut Cx, kind: OpKind, values: Vec<Value>) -> Result<String> {
        let op = DiscreteOp::new(kind);
        let value = op.call(cx, Args::new(values))?;
        match value.object().as_expr(cx)? {
            Expr::String(s) => Ok(s),
            other => panic!("expected string result, got {other:?}"),
        }
    }

    #[test]
    fn lib_installs_idempotently() {
        let mut cx = core_cx();
        install_discrete_lib(&mut cx).unwrap();
        install_discrete_lib(&mut cx).unwrap();
        assert!(cx.registry().lib(&manifest_name()).is_some());
    }

    #[test]
    fn combinatorics_ops() {
        let mut cx = core_cx();
        assert_eq!(
            call(&mut cx, OpKind::Binomial, vec!["10", "5"]).unwrap(),
            "252"
        );
        assert_eq!(call(&mut cx, OpKind::Factorial, vec!["6"]).unwrap(), "720");
        assert_eq!(
            call(&mut cx, OpKind::PartitionCount, vec!["5"]).unwrap(),
            "7"
        );
    }

    #[test]
    fn spectral_ops_round_trip_through_forms() {
        let mut cx = core_cx();
        let signal = forms::encode_fwht_signal(&[1, 0, 0, 0]);
        let out = call(&mut cx, OpKind::Fwht, vec![&signal]).unwrap();
        assert_eq!(forms::decode_fwht_signal(&out).unwrap(), vec![1, 1, 1, 1]);

        let a = forms::encode_fwht_signal(&[1, 2, 3, 4]);
        let b = forms::encode_fwht_signal(&[5, 6, 7, 8]);
        let conv = call(&mut cx, OpKind::XorConvolve, vec![&a, &b]).unwrap();
        // Spot-check against the naive XOR convolution component c[0].
        let c = forms::decode_fwht_signal(&conv).unwrap();
        // c[0] = sum_i a[i] * b[i ^ 0] = sum_i a[i] * b[i].
        let naive0: i64 = (0..4).map(|i| [1, 2, 3, 4][i] * [5, 6, 7, 8][i]).sum();
        assert_eq!(c[0], naive0);
    }

    #[test]
    fn mst_weight_op() {
        // Triangle weights: 0-1=1, 1-2=2, 0-2=3; MST weight = 3.
        let mut cx = core_cx();
        let adj = forms::encode_matrix(3, 3, &[0, 1, 3, 1, 0, 2, 3, 2, 0]);
        assert_eq!(call(&mut cx, OpKind::MstWeight, vec![&adj]).unwrap(), "3");
    }

    #[cfg(feature = "citizen")]
    #[test]
    fn runtime_ops_accept_citizen_values() {
        let mut cx = core_cx();

        let signal = FwhtSignalDescriptor::from_coeffs(&[1, 0, 0, 0]).unwrap();
        let signal_value = cx.factory().opaque(Arc::new(signal)).unwrap();
        let out = call_values(&mut cx, OpKind::Fwht, vec![signal_value]).unwrap();
        assert_eq!(forms::decode_fwht_signal(&out).unwrap(), vec![1, 1, 1, 1]);

        let matrix = MatrixDescriptor::from_parts(3, 3, &[0, 1, 3, 1, 0, 2, 3, 2, 0]).unwrap();
        let matrix_value = cx.factory().opaque(Arc::new(matrix)).unwrap();
        assert_eq!(
            call_values(&mut cx, OpKind::MstWeight, vec![matrix_value]).unwrap(),
            "3"
        );
    }

    #[test]
    fn bad_arity_is_an_error() {
        let mut cx = core_cx();
        assert!(call(&mut cx, OpKind::Binomial, vec!["5"]).is_err());
    }

    #[test]
    fn huge_combinatorics_inputs_are_bounded() {
        let mut cx = core_cx();
        // A huge factorial / partition argument must hit the input ceiling and
        // error, not allocate/iterate unboundedly.
        let err = call(&mut cx, OpKind::Factorial, vec!["99999999999"]).unwrap_err();
        assert!(err.to_string().contains("limit exceeded"), "{err}");
        let err = call(&mut cx, OpKind::PartitionCount, vec!["99999999999"]).unwrap_err();
        assert!(err.to_string().contains("limit exceeded"), "{err}");
        // A huge symmetric binomial (min(k, n - k) enormous) must be capped too,
        // not drive a ~5e11-iteration BigUint loop.
        let err = call(
            &mut cx,
            OpKind::Binomial,
            vec!["1000000000000", "500000000000"],
        )
        .unwrap_err();
        assert!(err.to_string().contains("limit exceeded"), "{err}");
        // The near-`n` edge (k just under n) collapses to a tiny loop and stays cheap.
        assert_eq!(
            call(
                &mut cx,
                OpKind::Binomial,
                vec!["1000000000000", "999999999999"]
            )
            .unwrap(),
            "1000000000000"
        );
    }
}
