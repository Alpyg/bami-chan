pub fn to_timestamp(t: u64) -> String {
    let sec = (t % 60) as u8;
    let min = ((t / 60) % 60) as u8;
    let hrs = t / 3600;

    return if hrs == 0 {
        format!("{:0>2}:{:0>2}", min, sec)
    } else {
        format!("{:0>2}:{:0>2}:{:0>2}", hrs, min, sec)
    };
}
