use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

const N: usize = 20;

fn main() -> std::io::Result<()> {
    let mut stream =
        TcpStream::connect("127.0.0.1:8080")?;

    println!("Conectado ao CalcServerTCP");

    let reader_stream = stream.try_clone()?;
    let mut reader = BufReader::new(reader_stream);

    let mut total_rtt = Duration::ZERO;
    let mut max_rtt = Duration::ZERO;
    let mut total_req_bytes = 0usize;
    let mut total_resp_bytes = 0usize;
    let sequence_start = Instant::now();

    for sequence in 0..N {
        let operand1 = sequence as f64 + 10.0;
        let operand2 = sequence as f64 + 1.0;

        let operation = match sequence % 4 {
            0 => "+",
            1 => "-",
            2 => "*",
            _ => "/",
        };

        let request = format!(
            "CALC:{}:{}:{}:{}",
            sequence,
            operand1,
            operation,
            operand2
        );

        // O protocolo textual envia a linha com '\n' (1 byte adicional)
        let req_bytes = request.len() + 1;
        total_req_bytes += req_bytes;

        println!("\nEnviando: {} ({} bytes)", request, req_bytes);

        let start = Instant::now();

        writeln!(stream, "{}", request)?;

        let mut response = String::new();

        reader.read_line(&mut response)?;

        let rtt = start.elapsed();

        let resp_bytes = response.len();
        total_resp_bytes += resp_bytes;

        total_rtt += rtt;
        if rtt > max_rtt {
            max_rtt = rtt;
        }

        println!(
            "Resposta: {} ({} bytes)",
            response.trim(),
            resp_bytes
        );

        println!(
            "RTT: {:.3} ms",
            rtt.as_secs_f64() * 1000.0
        );
    }

    let total_time = sequence_start.elapsed();

    let average_rtt =
        total_rtt.as_secs_f64() * 1000.0 / N as f64;
    let avg_req_bytes = total_req_bytes as f64 / N as f64;
    let avg_resp_bytes = total_resp_bytes as f64 / N as f64;
    let avg_total_msg_bytes = (total_req_bytes + total_resp_bytes) as f64 / (2 * N) as f64;

    println!("\n==============================");
    println!("Total de requisicoes: {}", N);
    println!(
        "Tempo total da sequencia: {:.3} ms",
        total_time.as_secs_f64() * 1000.0
    );
    println!(
        "RTT medio: {:.3} ms",
        average_rtt
    );
    println!(
        "RTT maximo: {:.3} ms",
        max_rtt.as_secs_f64() * 1000.0
    );
    println!("\n--- Tamanho das Mensagens (Texto) ---");
    println!("Tamanho medio por requisicao: {:.2} bytes", avg_req_bytes);
    println!("Tamanho medio por resposta:   {:.2} bytes", avg_resp_bytes);
    println!(
        "Tamanho medio global por mensagem: {:.2} bytes",
        avg_total_msg_bytes
    );
    println!(
        "Total de bytes trafegados: {} bytes",
        total_req_bytes + total_resp_bytes
    );

    Ok(())
}