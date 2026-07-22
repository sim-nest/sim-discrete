use sim_kernel::{
    Claim, ClaimPattern, Cx, Datum, LibId, Ref, Result, Symbol,
    card::{card_help_predicate, card_kind_predicate},
};

use crate::discrete_cards;

pub(crate) fn publish_discrete_card_claims_for_lib(cx: &mut Cx, lib_id: LibId) -> Result<()> {
    for card in discrete_cards() {
        publish_card(
            cx,
            lib_id,
            discrete_card_symbol(card.key),
            discrete_card_kind(),
            card.summary,
        )?;
    }

    #[cfg(feature = "rank")]
    for card in crate::rank::discrete_rank_cards() {
        publish_card(
            cx,
            lib_id,
            discrete_card_symbol(card.key),
            sim_kernel::rank::rank_space_kind(),
            card.summary,
        )?;
    }

    Ok(())
}

pub(crate) fn discrete_card_kind() -> Symbol {
    Symbol::qualified("discrete", "card")
}

pub(crate) fn discrete_card_symbol(key: &str) -> Symbol {
    match key.split_once('/') {
        Some((namespace, name)) => Symbol::qualified(namespace.to_owned(), name.to_owned()),
        None => Symbol::new(key.to_owned()),
    }
}

fn publish_card(
    cx: &mut Cx,
    lib_id: LibId,
    subject: Symbol,
    kind: Symbol,
    help: &str,
) -> Result<()> {
    let subject = Ref::Symbol(subject);
    insert_once(
        cx,
        lib_id,
        subject.clone(),
        card_kind_predicate(),
        Ref::Symbol(kind),
    )?;
    insert_string_once(cx, lib_id, subject, card_help_predicate(), help)
}

fn insert_once(
    cx: &mut Cx,
    lib_id: LibId,
    subject: Ref,
    predicate: Symbol,
    object: Ref,
) -> Result<()> {
    let exists = !cx
        .query_facts(ClaimPattern::exact(
            subject.clone(),
            predicate.clone(),
            object.clone(),
        ))?
        .is_empty();
    if !exists {
        cx.insert_fact_for_lib(lib_id, Claim::public(subject, predicate, object))?;
    }
    Ok(())
}

fn insert_string_once(
    cx: &mut Cx,
    lib_id: LibId,
    subject: Ref,
    predicate: Symbol,
    object: &str,
) -> Result<()> {
    let claim = Claim::content_object(
        cx.datum_store_mut(),
        subject.clone(),
        predicate.clone(),
        Datum::String(object.to_owned()),
    )?;
    insert_once(cx, lib_id, subject, predicate, claim.object)
}
