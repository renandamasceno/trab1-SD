use prost::Message;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

pub mod calculator {
    include!(concat!(env!("OUT_DIR"), "/calculator.rs"));
}

use calculator::calc_response::ResultOrError;
use calculator::{CalcRequest, CalcResponse, Operation};

fn process_request(request: CalcRequest) -> CalcResponse {
    let sequence = request.sequence;
    let op = Operation::try_from(request.op);

    match op {
        Ok(Operation::Add) => CalcResponse {
            sequence,
            result_or_error: Some(ResultOrError::Result(request.operand1 + request.operand2)),
        },
        Ok(Operation::Sub) => CalcResponse {
            sequence,
            result_or_error: Some(ResultOrError::Result(request.operand1 - request.operand2)),
        },
        Ok(Operation::Mul) => CalcResponse {
            sequence,
            result_or_error: Some(ResultOrError::Result(request.operand1 * request.operand2)),
        },
        Ok(Operation::Div) => {
            if request.operand2 == 0.0 {
                CalcResponse {
                    sequence,
                    result_or_error: Some(ResultOrError::Error("divisao por zero".to_string())),
                }
            } else {
                CalcResponse {
                    sequence,
                    result_or_error: Some(ResultOrError::Result(request.operand1 / request.operand2)),
                }
            }
        }
        Err(_) => CalcResponse {
            sequence,
            result_or_error: Some(ResultOrError::Error("operacao invalida".to_string())),
        },
    }
}

fn handle_client(mut stream: TcpStream) {
    let client_addr = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(_) => return,
    };

    println!("Cliente Proto conectado: {}", client_addr);

    loop {
        let mut len_buf = [0u8; 4];
        match stream.read_exact(&mut len_buf) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
                println!("Cliente {} desconectou", client_addr);
                break;
            }
            Err(error) => {
                eprintln!("Erro ao ler tamanho de {}: {}", client_addr, error);
                break;
            }
        }

        let len = u32::from_be_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        if let Err(error) = stream.read_exact(&mut buf) {
            eprintln!("Erro ao ler payload de {}: {}", client_addr, error);
            break;
        }

        let request = match CalcRequest::decode(&buf[..]) {
            Ok(req) => req,
            Err(error) => {
                eprintln!("Erro ao decodificar Protobuf de {}: {}", client_addr, error);
                break;
            }
        };

        println!(
            "[{}] Requisição #{}: op1={}, op={:?}, op2={} ({} bytes)",
            client_addr,
            request.sequence,
            request.operand1,
            Operation::try_from(request.op),
            request.operand2,
            len
        );

        let response = process_request(request);

        let mut resp_buf = Vec::new();
        if let Err(error) = response.encode(&mut resp_buf) {
            eprintln!("Erro ao serializar resposta para {}: {}", client_addr, error);
            break;
        }

        let resp_len = resp_buf.len() as u32;
        if let Err(error) = stream.write_all(&resp_len.to_be_bytes()) {
            eprintln!("Erro ao enviar tamanho para {}: {}", client_addr, error);
            break;
        }
        if let Err(error) = stream.write_all(&resp_buf) {
            eprintln!("Erro ao enviar resposta para {}: {}", client_addr, error);
            break;
        }
        if let Err(error) = stream.flush() {
            eprintln!("Erro ao descarregar buffer para {}: {}", client_addr, error);
            break;
        }

        println!(
            "[{}] Resposta #{}: {:?} ({} bytes)",
            client_addr,
            response.sequence,
            response.result_or_error,
            resp_buf.len()
        );
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    println!("CalcServerProto escutando em 127.0.0.1:8080 (TCP + Protobuf)");

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(error) => {
                eprintln!("Erro ao aceitar conexão: {}", error);
            }
        }
    }

    Ok(())
}
