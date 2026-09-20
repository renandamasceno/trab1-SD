use rand::RngExt;
use std::env;
use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;

fn process_request(request: &str) -> String {
    let parts: Vec<&str> = request.trim().split(':').collect();

    if parts.len() != 5 || parts[0] != "CALC" {
        return "ERROR:0:formato invalido".to_string();
    }

    let sequence = parts[1];

    let operand1: f64 = match parts[2].parse() {
        Ok(value) => value,
        Err(_) => {
            return format!(
                "ERROR:{}:operando1 invalido",
                sequence
            );
        }
    };

    let operation = parts[3];

    let operand2: f64 = match parts[4].parse() {
        Ok(value) => value,
        Err(_) => {
            return format!(
                "ERROR:{}:operando2 invalido",
                sequence
            );
        }
    };

    match operation {
        "+" => format!(
            "RESULT:{}:{}",
            sequence,
            operand1 + operand2
        ),

        "-" => format!(
            "RESULT:{}:{}",
            sequence,
            operand1 - operand2
        ),

        "*" => format!(
            "RESULT:{}:{}",
            sequence,
            operand1 * operand2
        ),

        "/" => {
            if operand2 == 0.0 {
                format!(
                    "ERROR:{}:divisao por zero",
                    sequence
                )
            } else {
                format!(
                    "RESULT:{}:{}",
                    sequence,
                    operand1 / operand2
                )
            }
        }

        _ => format!(
            "ERROR:{}:operacao invalida",
            sequence
        ),
    }
}

fn get_loss_rate() -> f64 {
    let args: Vec<String> = env::args().collect();

    if let Some(index) =
        args.iter().position(|arg| arg == "--loss-rate")
    {
        if let Some(value) = args.get(index + 1) {
            return value.parse::<f64>().unwrap_or(0.0);
        }
    }

    0.0
}

fn main() -> std::io::Result<()> {
    let loss_rate = get_loss_rate();

    if !(0.0..=1.0).contains(&loss_rate) {
        eprintln!("loss-rate deve estar entre 0.0 e 1.0");
        return Ok(());
    }

    let socket =
        Arc::new(UdpSocket::bind("127.0.0.1:8080")?);

    println!("CalcServerUDP em 127.0.0.1:8080");
    println!("Taxa de perda simulada: {:.0}%", loss_rate * 100.0);

    loop {
        let mut buffer = [0u8; 1024];

        let (bytes_read, client_addr) =
            socket.recv_from(&mut buffer)?;

        /*
         * Precisamos copiar os bytes porque buffer pertence
         * a esta iteração do loop.
         */
        let data =
            buffer[..bytes_read].to_vec();

        let socket = Arc::clone(&socket);

        thread::spawn(move || {
            let request =
                String::from_utf8_lossy(&data);

            println!(
                "\n[{}] Recebido: {}",
                client_addr,
                request
            );

            /*
             * Simulação de perda.
             *
             * Exemplo:
             *
             * loss_rate = 0.1
             *
             * aproximadamente 10% dos datagramas
             * recebidos não terão resposta.
             */
            let mut rng = rand::rng();

            let lost =
                rng.random::<f64>() < loss_rate;

            if lost {
                println!(
                    "[{}] PACOTE DESCARTADO",
                    client_addr
                );

                return;
            }

            let response =
                process_request(&request);

            match socket.send_to(
                response.as_bytes(),
                client_addr,
            ) {
                Ok(_) => {
                    println!(
                        "[{}] Enviado: {}",
                        client_addr,
                        response
                    );
                }

                Err(error) => {
                    eprintln!(
                        "[{}] Erro: {}",
                        client_addr,
                        error
                    );
                }
            }
        });
    }
}