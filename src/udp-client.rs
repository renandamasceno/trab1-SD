use rand::RngExt;
use std::io;
use std::net::UdpSocket;
use std::time::{Duration, Instant};

const N: usize = 20;

const TIMEOUT_MS: u64 = 500;

const MAX_ATTEMPTS: usize = 5;

fn generate_request(sequence: usize) -> String {
    let mut rng = rand::rng();

    let operand1: f64 =
        rng.random_range(1.0..100.0);

    let operand2: f64 =
        rng.random_range(1.0..100.0);

    let operations = ["+", "-", "*", "/"];

    let operation =
        operations[rng.random_range(0..operations.len())];

    format!(
        "CALC:{}:{:.2}:{}:{:.2}",
        sequence,
        operand1,
        operation,
        operand2
    )
}

fn main() -> std::io::Result<()> {
    let socket =
        UdpSocket::bind("127.0.0.1:0")?;

    let server_addr = "127.0.0.1:8080";

    socket.set_read_timeout(
        Some(Duration::from_millis(TIMEOUT_MS))
    )?;

    println!(
        "CalcClientUDP usando {}",
        socket.local_addr()?
    );

    println!(
        "Timeout: {} ms",
        TIMEOUT_MS
    );

    println!(
        "Máximo de tentativas: {}",
        MAX_ATTEMPTS
    );

    let mut received = 0;
    let mut lost = 0;

    let mut total_rtt =
        Duration::ZERO;
    let mut max_rtt =
        Duration::ZERO;

    let mut total_transmissions = 0;
    let sequence_start = Instant::now();

    for sequence in 0..N {
        let request =
            generate_request(sequence);

        println!(
            "\n================================="
        );

        println!(
            "Requisição #{}: {}",
            sequence,
            request
        );

        let mut success = false;

        /*
         * O RTT é medido desde a transmissão que
         * efetivamente recebeu resposta.
         *
         * Se quiser medir incluindo retransmissões,
         * basta mover o Instant para fora deste loop.
         */
        for attempt in 1..=MAX_ATTEMPTS {
            println!(
                "Tentativa {}/{}",
                attempt,
                MAX_ATTEMPTS
            );

            let start =
                Instant::now();

            socket.send_to(
                request.as_bytes(),
                server_addr,
            )?;

            total_transmissions += 1;

            let mut buffer =
                [0u8; 1024];

            match socket.recv_from(&mut buffer) {
                Ok((bytes_read, source)) => {
                    let rtt =
                        start.elapsed();

                    let response =
                        String::from_utf8_lossy(
                            &buffer[..bytes_read]
                        );

                    println!(
                        "Resposta de {}: {}",
                        source,
                        response
                    );

                    println!(
                        "RTT: {:.3} ms",
                        rtt.as_secs_f64() * 1000.0
                    );

                    total_rtt += rtt;
                    if rtt > max_rtt {
                        max_rtt = rtt;
                    }

                    received += 1;

                    success = true;

                    break;
                }

                Err(error)
                if error.kind()
                    == io::ErrorKind::WouldBlock
                    || error.kind()
                    == io::ErrorKind::TimedOut =>
                    {
                        println!(
                            "Timeout após {} ms.",
                            TIMEOUT_MS
                        );

                        if attempt < MAX_ATTEMPTS {
                            println!(
                                "Retransmitindo..."
                            );
                        }
                    }

                Err(error) => {
                    return Err(error);
                }
            }
        }

        if !success {
            println!(
                "REQUISIÇÃO #{} PERDIDA após {} tentativas",
                sequence,
                MAX_ATTEMPTS
            );

            lost += 1;
        }
    }

    let total_time = sequence_start.elapsed();

    println!();
    println!("=================================");
    println!("RESULTADOS");
    println!("=================================");

    println!(
        "Requisições: {}",
        N
    );

    println!(
        "Respondidas: {}",
        received
    );

    println!(
        "Perdidas: {}",
        lost
    );

    println!(
        "Datagramas enviados: {}",
        total_transmissions
    );

    println!(
        "Retransmissões: {}",
        total_transmissions - N
    );

    println!(
        "Tempo total da sequência: {:.3} ms",
        total_time.as_secs_f64() * 1000.0
    );

    if received > 0 {
        let average_rtt =
            total_rtt.as_secs_f64()
                * 1000.0
                / received as f64;

        println!(
            "RTT médio: {:.3} ms",
            average_rtt
        );

        println!(
            "RTT máximo: {:.3} ms",
            max_rtt.as_secs_f64() * 1000.0
        );
    }

    Ok(())
}