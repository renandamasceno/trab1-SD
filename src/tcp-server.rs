use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
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
            return format!("ERROR:{}:operando1 invalido", sequence);
        }
    };

    let operation = parts[3];

    let operand2: f64 = match parts[4].parse() {
        Ok(value) => value,
        Err(_) => {
            return format!("ERROR:{}:operando2 invalido", sequence);
        }
    };

    match operation {
        "+" => format!("RESULT:{}:{}", sequence, operand1 + operand2),
        "-" => format!("RESULT:{}:{}", sequence, operand1 - operand2),
        "*" => format!("RESULT:{}:{}", sequence, operand1 * operand2),

        "/" => {
            if operand2 == 0.0 {
                format!("ERROR:{}:divisao por zero", sequence)
            } else {
                format!("RESULT:{}:{}", sequence, operand1 / operand2)
            }
        }

        _ => format!("ERROR:{}:operacao invalida", sequence),
    }
}

fn handle_client(mut stream: TcpStream) {
    let client_addr = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(_) => return,
    };

    println!("Cliente conectado: {}", client_addr);

    let reader_stream = match stream.try_clone() {
        Ok(stream) => stream,
        Err(error) => {
            eprintln!("Erro ao clonar stream: {}", error);
            return;
        }
    };

    let mut reader = BufReader::new(reader_stream);

    loop {
        let mut request = String::new();

        match reader.read_line(&mut request) {
            Ok(0) => {
                println!("Cliente {} desconectou", client_addr);
                break;
            }

            Ok(_) => {
                let request = request.trim();

                println!(
                    "[{}] Requisicao recebida: {}",
                    client_addr,
                    request
                );

                let response = process_request(request);

                println!(
                    "[{}] Resposta: {}",
                    client_addr,
                    response
                );

                if let Err(error) =
                    writeln!(stream, "{}", response)
                {
                    eprintln!(
                        "Erro ao enviar resposta para {}: {}",
                        client_addr,
                        error
                    );

                    break;
                }
            }

            Err(error) => {
                eprintln!(
                    "Erro ao ler dados de {}: {}",
                    client_addr,
                    error
                );

                break;
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    println!(
        "CalcServerTCP escutando em 127.0.0.1:8080"
    );

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }

            Err(error) => {
                eprintln!(
                    "Erro ao aceitar conexao: {}",
                    error
                );
            }
        }
    }

    Ok(())
}