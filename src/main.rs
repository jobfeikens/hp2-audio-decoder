extern crate core;

use std::{env, fs};
use std::fs::File;
use std::io::{Write};

const FRAME_SIZE: usize = 15;

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

fn decode_frame(context: &mut Context, frame: &[u8]) -> Vec<i16> {
    let header = decode_frame_header(&frame);
    let mut buffer = vec![];
    for i in 1..FRAME_SIZE {
        let nibbles = [
            (frame[i] >> 4),
            (frame[i] & 0x0F)
        ];
        for nibble in nibbles {
            let sample = decode_sample(context, nibble, &header);
            context.sample_history_2 = context.sample_history_1;
            context.sample_history_1 = sample;
            buffer.push(sample);
        }
    }
    buffer
}

fn decode_frame_header(frame: &[u8]) -> FrameHeader {
    let header_byte = frame[0];
    let coefficients = EA_XA_ADPCM_TABLE[(header_byte >> 4) as usize];
    FrameHeader {
        coefficient_1: coefficients[0],
        coefficient_2: coefficients[1],
        shift: (header_byte & 0x0F) + 8,
    }
}

fn decode_sample(context: &Context, nibble: u8, header: &FrameHeader) -> i16 {
    let sample = (((nibble as i32) << 28 >> header.shift)
        + (header.coefficient_1 as i32 * context.sample_history_1 as i32)
        + (header.coefficient_2 as i32 * context.sample_history_2 as i32))
        >> 8;
    sample as i16
}

fn main() -> anyhow::Result<()> {
    let args = env::args().collect::<Vec<String>>();
    let input_path = args.get(1).expect("Enter input path");
    let output_path = args.get(2).expect("Enter output path");

    let mut context = Context::new();
    let bytes = fs::read(input_path)?;

    let mut output_buffer = vec![];

    for frame in bytes.chunks(FRAME_SIZE) {
        output_buffer.extend(decode_frame(&mut context, &frame));
    }

    let mut output_buffer_u8: Vec<u8> = vec![];

    for sample in output_buffer {
        output_buffer_u8.push((sample & 0xFF) as u8);
        output_buffer_u8.push((sample >> 8) as u8);
    }

    let mut output_file = File::create(output_path)?;

    output_file.write_all(&output_buffer_u8)?;

    Ok(())
}
