use crate::component::{Entity, EntityMap};

pub fn zip3_entities_mut<'a, T, U, V>(
    a: &'a mut EntityMap<T>,
    b: &'a mut EntityMap<U>,
    c: &'a mut EntityMap<V>,
) -> impl Iterator<
    Item = (
        Entity,
        &'a mut Option<T>,
        &'a mut Option<U>,
        &'a mut Option<V>,
    ),
> {
    a.iter_mut()
        .zip(b.iter_mut())
        .zip(c.iter_mut())
        .map(|(((entity, a_i), (_, b_i)), (_, c_i))| (entity, a_i, b_i, c_i))
}

pub fn zip3_entities_1immut<'a, T, U, V>(
    a: &'a mut EntityMap<T>,
    b: &'a mut EntityMap<U>,
    c: &'a EntityMap<V>,
) -> impl Iterator<Item = (Entity, &'a mut Option<T>, &'a mut Option<U>, &'a Option<V>)> {
    a.iter_mut()
        .zip(b.iter_mut())
        .zip(c.iter())
        .map(|(((entity, a_i), (_, b_i)), (_, c_i))| (entity, a_i, b_i, c_i))
}

pub fn zip3_entities<'a, T, U, V>(
    a: &'a EntityMap<T>,
    b: &'a EntityMap<U>,
    c: &'a EntityMap<V>,
) -> impl Iterator<Item = (Entity, &'a Option<T>, &'a Option<U>, &'a Option<V>)> {
    a.iter()
        .zip(b.iter())
        .zip(c.iter())
        .map(|(((entity, a_i), (_, b_i)), (_, c_i))| (entity, a_i, b_i, c_i))
}

pub fn zip4_entities_mut<'a, T, U, V, W>(
    a: &'a mut EntityMap<T>,
    b: &'a mut EntityMap<U>,
    c: &'a mut EntityMap<V>,
    d: &'a mut EntityMap<W>,
) -> impl Iterator<
    Item = (
        Entity,
        &'a mut Option<T>,
        &'a mut Option<U>,
        &'a mut Option<V>,
        &'a mut Option<W>,
    ),
> {
    a.iter_mut()
        .zip(b.iter_mut())
        .zip(c.iter_mut())
        .zip(d.iter_mut())
        .map(|((((entity, a_i), (_, b_i)), (_, c_i)), (_, d_i))| (entity, a_i, b_i, c_i, d_i))
}

pub fn zip4_entities_1immut<'a, T, U, V, W>(
    a: &'a mut EntityMap<T>,
    b: &'a mut EntityMap<U>,
    c: &'a mut EntityMap<V>,
    d: &'a EntityMap<W>,
) -> impl Iterator<
    Item = (
        Entity,
        &'a mut Option<T>,
        &'a mut Option<U>,
        &'a mut Option<V>,
        &'a Option<W>,
    ),
> {
    a.iter_mut()
        .zip(b.iter_mut())
        .zip(c.iter_mut())
        .zip(d.iter())
        .map(|((((entity, a_i), (_, b_i)), (_, c_i)), (_, d_i))| (entity, a_i, b_i, c_i, d_i))
}

pub fn zip4_entities<'a, T, U, V, W>(
    a: &'a EntityMap<T>,
    b: &'a EntityMap<U>,
    c: &'a EntityMap<V>,
    d: &'a EntityMap<W>,
) -> impl Iterator<
    Item = (
        Entity,
        &'a Option<T>,
        &'a Option<U>,
        &'a Option<V>,
        &'a Option<W>,
    ),
> {
    a.iter()
        .zip(b.iter())
        .zip(c.iter())
        .zip(d.iter())
        .map(|((((entity, a_i), (_, b_i)), (_, c_i)), (_, d_i))| (entity, a_i, b_i, c_i, d_i))
}

pub fn zip5_entities_1immut<'a, T, U, V, W, X>(
    a: &'a mut EntityMap<T>,
    b: &'a mut EntityMap<U>,
    c: &'a mut EntityMap<V>,
    d: &'a mut EntityMap<W>,
    e: &'a EntityMap<X>,
) -> impl Iterator<
    Item = (
        Entity,
        &'a mut Option<T>,
        &'a mut Option<U>,
        &'a mut Option<V>,
        &'a mut Option<W>,
        &'a Option<X>,
    ),
> {
    a.iter_mut()
        .zip(b.iter_mut())
        .zip(c.iter_mut())
        .zip(d.iter_mut())
        .zip(e.iter())
        .map(
            |(((((entity, a_i), (_, b_i)), (_, c_i)), (_, d_i)), (_, e_i))| {
                (entity, a_i, b_i, c_i, d_i, e_i)
            },
        )
}
