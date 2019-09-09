pub(crate) fn lerp_to_vec<M>(start: M, end: M, length: usize) -> Vec<M>
where
    M: std::ops::Mul<f32, Output = M> + std::ops::Add<M, Output = M> + Clone + std::fmt::Debug,
{
    let mut v = Vec::with_capacity(length);

    for i in 0..length {
        let rate = (i as f32) / (length as f32);
        let val = start.clone() * (1.0 - rate) + end.clone() * rate;
        v.push(val);
    }

    v
}
