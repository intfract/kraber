pub fn add(nums: Vec<f64>) -> f64 {
    let mut sum: f64 = 0.0;
    for num in nums {
        sum += num;
    }
    return sum;
}