use std::fs;
use std::fs::File;
use std::io::{BufRead, Read, Write};
use std::os::unix::prelude::FileExt;

const EA_XA_ADPCM_TABLE: [[i16; 2]; 4] = [
    [0, 0],
    [240, 0],
    [460, -208],
    [392, -220]
];

struct Context {
    sample_history_1: i16,
    sample_history_2: i16,
}

impl Context {
    fn new() -> Self {
        Context {
            sample_history_1: 0,
            sample_history_2: 0,
        }
    }
}

struct FrameHeader {
    coefficient_1: i16,
    coefficient_2: i16,
    shift: u8,
}

impl FrameHeader {
    fn from_byte(byte: u8) -> Self {
        let [coefficient_1, coefficient_2] = EA_XA_ADPCM_TABLE[(byte >> 4) as usize];

        FrameHeader {
            coefficient_1,
            coefficient_2,
            shift: (byte & 0x0F) + 8,
        }
    }
}

fn decode_frame(context: &mut Context, frame: &[u8], output_buffer: &mut Vec<i16>) {
    let header = FrameHeader::from_byte(frame[0]);

    for i in 0..14 {
        let samples = get_nibbles(&frame[1 + i]);

        for sample in samples {
            output_buffer.push(decode_sample(context, sample, &header));
        }
    }
}

fn decode_sample(context: &mut Context, sample: u8, header: &FrameHeader) -> i16 {
    let decoded_sample = ((((sample as i32) << 28 >> header.shift)
        + (header.coefficient_1 as i32 * context.sample_history_1 as i32)
        + (header.coefficient_2 as i32 * context.sample_history_2 as i32))
        >> 8) as i16;

    context.sample_history_2 = context.sample_history_1;
    context.sample_history_1 = decoded_sample;

    return decoded_sample;
}


fn get_nibbles(byte: &u8) -> [u8; 2] {
    [byte >> 4, byte & 0x0F]
}

fn main() -> anyhow::Result<()> {
    let mut context = Context::new();
    let file = File::open("sample.Sound")?;
    let metadata = fs::metadata("sample.Sound")?;
    let mut output_buffer = vec![];

    for offset in (33..metadata.len() - 1500).step_by(15) {
        println!("{}", offset);

        if metadata.len() - offset < 15 {
            break;
        }

        let mut frame = vec![0u8; 15];

        file.read_at(&mut frame, offset)?;
        decode_frame(&mut context, &frame, &mut output_buffer);
    }

    let mut output_buffer_u8: Vec<u8> = vec![];

    for sample in output_buffer {
        output_buffer_u8.push((sample & 0xFF) as u8);
        output_buffer_u8.push((sample >> 8) as u8);
    }

    let mut output_file = File::create("output.bin")?;

    output_file.write_all(&output_buffer_u8)?;

    Ok(())
}
