pub fn avatar(v: char) -> Vec<u8> {
    match v {
        // 0123456789ABCDEFGHJKMNPQRSTVWXYZ
        '0' | '1' | '2' | '3' | 'S' | 'Z' => include_bytes!(crate::asset!("user/2.png")).to_vec(),
        '4' | '5' | '6' | '7' | 'T' => include_bytes!(crate::asset!("user/3.png")).to_vec(),
        '8' | '9' | 'A' | 'B' => include_bytes!(crate::asset!("user/4.png")).to_vec(),
        'C' | 'D' | 'E' | 'F' | 'V' => include_bytes!(crate::asset!("user/5.png")).to_vec(),
        'G' | 'H' | 'J' | 'K' | 'W' => include_bytes!(crate::asset!("user/6.png")).to_vec(),
        'M' | 'N' | 'P' | 'Q' | 'X' => include_bytes!(crate::asset!("user/7.png")).to_vec(),
        /*'0' | '1' | '2' | '3' | 'R' | 'Y'*/
        _ => include_bytes!(crate::asset!("user/1.png")).to_vec(),
    }
}
