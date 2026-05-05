/// IPC message decompression.
///
/// Ported from the JavaScript `decompress` function in ipc.js.
/// Implements the kdb+ IPC compression scheme.

pub fn decompress(c_msg: &[u8]) -> Vec<u8> {
    let o_len = i32::from_le_bytes([c_msg[8], c_msg[9], c_msg[10], c_msg[11]]) as usize;
    let mut msg = vec![0u8; o_len];

    // Copy header bytes [0..4]
    msg[..4].copy_from_slice(&c_msg[..4]);
    // Clear compression flag
    msg[2] = 0;
    // Copy original length into msg length field [4..8]
    msg[4..8].copy_from_slice(&c_msg[8..12]);

    let mut c_pos: usize = 12;
    let mut o_pos: usize = 8;
    let mut x_pos: usize = o_pos;
    let mut n: u8 = 0;
    let mut s: usize;
    let mut r: usize;
    let mut i: u16 = 0;
    let mut x = [0i32; 256];

    while o_pos < o_len {
        if i == 0 {
            n = c_msg[c_pos];
            c_pos += 1;
            i = 1;
        }

        r = 0;
        if (n & (i as u8)) != 0 {
            s = x[c_msg[c_pos] as usize] as usize;
            c_pos += 1;
            r = c_msg[c_pos] as usize;
            c_pos += 1;
            for j in 0..r + 2 {
                msg[o_pos + j] = msg[s + j];
            }
            o_pos += 2;
        } else {
            msg[o_pos] = c_msg[c_pos];
            o_pos += 1;
            c_pos += 1;
        }

        while x_pos < o_pos.saturating_sub(1) {
            let idx = (msg[x_pos] ^ msg[x_pos + 1]) as usize;
            x[idx] = x_pos as i32;
            x_pos += 1;
        }
        if (n & (i as u8)) != 0 {
            o_pos += r;
            x_pos = o_pos;
        }
        i *= 2;
        if i == 256 {
            i = 0;
        }
    }
    msg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress_boolean_list() {
        // Compressed message representing 2000 `true` values
        let compressed = hex::decode(
            "0110010026000000de070000000100d00700000101ff00ff00ff00ff00ff00ff00ff00ff00c5",
        )
        .unwrap();
        let decompressed = decompress(&compressed);

        // Header: kType=1 (boolean list), attr=0, length=2000
        assert_eq!(decompressed[8], 1); // kType
        let len = i32::from_le_bytes([
            decompressed[10],
            decompressed[11],
            decompressed[12],
            decompressed[13],
        ]);
        assert_eq!(len, 2000);

        // All values should be 1 (true)
        for i in 14..14 + 2000 {
            assert_eq!(decompressed[i], 1, "byte at position {i} should be 1");
        }
    }
}
