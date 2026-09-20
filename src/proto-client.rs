use prost::Message;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

pub mod calculator {
    include!(concat!(env!("OUT_DIR"), "/calculator.rs"));
}

use calculator::calc_response::ResultOrError;
use calculator::{CalcRequest, CalcResponse, Operation};

const N: usize = 20;

fn main() -> std::io::Result<()> {
    let mut stream = match TcpStream::connect("127.0.0.1:8080") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Erro ao conectar a 127.0.0.1:8080: {}", e);
            return Err(e);
        }
    };

    println!("Conectado ao CalcServerProto (TCP + Protobuf)");

    let mut total_rtt = Duration::ZERO;
    let mut max_rtt = Duration::ZERO;
    let mut total_req_bytes = 0usize;
    let mut total_resp_bytes = 0usize;

    let sequence_start = Instant::now();

    for sequence in 0..N {
        let operand1 = sequence as f64 + 10.0;
        let operand2 = sequence as f64 + 1.0;

        let op = match sequence % 4 {
            0 => Operation::Add,
            1 => Operation::Sub,
            2 => Operation::Mul,
            _ => Operation::Div,
        };

        let request = CalcRequest {
            sequence: sequence as u32,
            operand1,
            op: op as i32,
            operand2,
        };

        let mut req_buf = Vec::new();
        request.encode(&mut req_buf)?;
        let req_len = req_buf.len();
        total_req_bytes += req_len;

        println!(
            "\nEnviando #{}: op1={}, op={:?}, op2={} ({} bytes protobuf)",
            sequence, operand1, op, operand2, req_len
        );

        let start = Instant::now();

        // Enviar tamanho (4 bytes big-endian) + payload protobuf
        let req_len_u32 = req_len as u32;
        stream.write_all(&req_len_u32.to_be_bytes())?;
        stream.write_all(&req_buf)?;
        stream.flush()?;

        // Ler tamanho da resposta
        let mut resp_len_buf = [0u8; 4];
        stream.read_exact(&mut resp_len_buf)?;
        let resp_len = u32::from_be_bytes(resp_len_buf) as usize;
        total_resp_bytes += resp_len;

        // Ler payload da resposta
        let mut resp_buf = vec![0u8; resp_len];
        stream.read_exact(&mut resp_buf)?;

        let rtt = start.elapsed();
        total_rtt += rtt;
        if rtt > max_rtt {
            max_rtt = rtt;
        }

        let response = CalcResponse::decode(&resp_buf[..])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        match response.result_or_error {
            Some(ResultOrError::Result(val)) => {
                println!(
                    "Resposta #{}: RESULT = {} ({} bytes protobuf)",
                    response.sequence, val, resp_len
                );
            }
            Some(ResultOrError::Error(msg)) => {
                println!(
                    "Resposta #{}: ERROR = {} ({} bytes protobuf)",
                    response.sequence, msg, resp_len
                );
            }
            None => {
                println!(
                    "Resposta #{}: Resposta vazia ({} bytes)",
                    response.sequence, resp_len
                );
            }
        }

        println!("RTT: {:.3} ms", rtt.as_secs_f64() * 1000.0);
    }

    let total_time = sequence_start.elapsed();
    let average_rtt = total_rtt.as_secs_f64() * 1000.0 / N as f64;
    let avg_req_bytes = total_req_bytes as f64 / N as f64;
    let avg_resp_bytes = total_resp_bytes as f64 / N as f64;
    let avg_total_msg_bytes = (total_req_bytes + total_resp_bytes) as f64 / (2 * N) as f64;

    // Teste explícito de tratamento de erro: divisão por zero
    println!("\n--- Teste de Tratamento de Erro (Divisão por Zero) ---");
    let err_request = CalcRequest {
        sequence: 999,
        operand1: 42.0,
        op: Operation::Div as i32,
        operand2: 0.0,
    };
    let mut err_buf = Vec::new();
    err_request.encode(&mut err_buf)?;
    let err_len = err_buf.len() as u32;
    stream.write_all(&err_len.to_be_bytes())?;
    stream.write_all(&err_buf)?;
    stream.flush()?;

    let mut err_resp_len_buf = [0u8; 4];
    stream.read_exact(&mut err_resp_len_buf)?;
    let err_resp_len = u32::from_be_bytes(err_resp_len_buf) as usize;
    let mut err_resp_buf = vec![0u8; err_resp_len];
    stream.read_exact(&mut err_resp_buf)?;
    let err_response = CalcResponse::decode(&err_resp_buf[..])
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    match err_response.result_or_error {
        Some(ResultOrError::Error(msg)) => {
            println!("Servidor retornou erro com sucesso: '{}'", msg);
        }
        other => {
            println!("Resposta inesperada: {:?}", other);
        }
    }

    println!("\n=================================");
    println!("RESULTADOS PROTOBUF");
    println!("=================================");
    println!("Total de requisições: {}", N);
    println!(
        "Tempo total da sequência: {:.3} ms",
        total_time.as_secs_f64() * 1000.0
    );
    println!("RTT médio: {:.3} ms", average_rtt);
    println!("RTT máximo: {:.3} ms", max_rtt.as_secs_f64() * 1000.0);
    println!("\n--- Tamanho das Mensagens (Bytes) ---");
    println!("Tamanho médio por requisição: {:.2} bytes", avg_req_bytes);
    println!("Tamanho médio por resposta:   {:.2} bytes", avg_resp_bytes);
    println!(
        "Tamanho médio global por mensagem: {:.2} bytes",
        avg_total_msg_bytes
    );
    println!(
        "Total de bytes trafegados (payload): {} bytes",
        total_req_bytes + total_resp_bytes
    );
    println!(
        "Total com framing (4 bytes prefixo por msg): {} bytes",
        total_req_bytes + total_resp_bytes + (2 * N * 4)
    );

    Ok(())
}
