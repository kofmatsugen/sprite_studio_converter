pub(crate) fn lerp_to_vec(start: f32, end: f32, length: usize) -> Vec<f32> {
    let mut v = Vec::with_capacity(length);

    for i in 0..length {
        let rate = (i as f32) / (length as f32);
        let val = start * (1.0 - rate) + end * rate;
        v.push(val);
    }

    v
}
