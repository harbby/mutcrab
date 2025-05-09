const MAXIMUM_CAPACITY: usize = 1 << 31;

pub fn next_power_of_two(num: usize) -> usize {
    if num == 0 {
        return 1;
    }
    let mut value = num - 1;
    value |= value >> 1;
    value |= value >> 2;
    value |= value >> 4;
    value |= value >> 8;
    value |= value >> 16;
    // return value + 1;
    if value > MAXIMUM_CAPACITY {
        MAXIMUM_CAPACITY
    } else {
        value + 1
    }
}