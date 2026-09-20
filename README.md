# Trabalho 1 — Comunicação TCP e UDP em Rust

Este projeto implementa uma calculadora remota utilizando os protocolos TCP e UDP em Rust.

O objetivo é comparar o comportamento dos dois protocolos, especialmente em relação a confiabilidade, retransmissão, ordenação e impacto de perdas simuladas.

## Estrutura do projeto

```text
├── Cargo.toml
├── build.rs
├── proto/
│   └── calculator.proto
└── src/
    ├── tcp-client.rs
    ├── tcp-server.rs
    ├── udp-client.rs
    ├── udp-server.rs
    ├── proto-client.rs
    └── proto-server.rs
```

Os binários disponíveis são:

```text
client        (TCP textual)
server        (TCP textual)
client-udp    (UDP com confiabilidade)
server-udp    (UDP com descarte simulado)
client-proto  (TCP binário com Protocol Buffers)
server-proto  (TCP binário com Protocol Buffers)
```

## Protocolo da calculadora

As requisições seguem o formato:

```text
CALC:<n>:<operando1>:<op>:<operando2>
```

Onde:

* `n` é o número de sequência da requisição;
* `operando1` e `operando2` são valores numéricos;
* `op` pode ser `+`, `-`, `*` ou `/`.

Exemplo:

```text
CALC:0:10:+:5
```

Resposta:

```text
RESULT:0:15
```

Em caso de erro:

```text
ERROR:<n>:<mensagem>
```

Exemplo:

```text
CALC:1:10:/:0
```

Resposta:

```text
ERROR:1:divisao por zero
```

## Implementação TCP

### Servidor TCP

O servidor TCP:

* escuta conexões na porta `8080`;
* aceita múltiplos clientes;
* cria uma thread independente para cada cliente;
* recebe requisições de cálculo;
* executa a operação;
* retorna o resultado ao cliente.

Como TCP é um fluxo de bytes, as mensagens utilizam `\n` como delimitador.

### Cliente TCP

O cliente TCP:

* estabelece conexão com o servidor;
* envia `N = 20` requisições numeradas;
* aguarda a resposta antes de enviar a próxima;
* mede o RTT de cada requisição;
* não implementa retransmissão na aplicação.

O TCP fornece mecanismos de confiabilidade, ordenação e retransmissão automaticamente.

### Executando

Servidor:

```bash
cargo run --bin server
```

Cliente:

```bash
cargo run --bin client
```

## Implementação UDP

### Servidor UDP

O servidor UDP:

* escuta datagramas na porta `8080`;
* recebe requisições de múltiplos clientes;
* executa as operações matemáticas;
* envia a resposta para o endereço de origem;
* permite configurar uma taxa de perda simulada.

A perda simulada representa uma falha de omissão: o servidor recebe o datagrama, mas propositalmente não envia uma resposta.

### Executando com perda simulada

Sem perda:

```bash
cargo run --bin server-udp -- --loss-rate 0.0
```

Com 10% de perda:

```bash
cargo run --bin server-udp -- --loss-rate 0.1
```

Com 30% de perda:

```bash
cargo run --bin server-udp -- --loss-rate 0.3
```

Exemplo com 90% de perda para testes:

```bash
cargo run --bin server-udp -- --loss-rate 0.9
```

Cliente:

```bash
cargo run --bin client-udp
```

O argumento `--loss-rate` deve ser passado ao servidor UDP, pois é ele que simula o descarte das mensagens.

## Confiabilidade sobre UDP

UDP não garante:

* entrega;
* ordenação;
* retransmissão;
* confirmação de recebimento.

Por isso, o cliente implementa um mecanismo simples de confiabilidade.

Para cada requisição:

```text
envia requisição
       |
       v
aguarda resposta
       |
       v
 resposta chegou?
   /           \
 sim            não
  |              |
mede RTT       timeout
                 |
                 v
             retransmite
```

O timeout utilizado é de:

```text
500 ms
```

O máximo de tentativas é:

```text
5
```

Caso nenhuma resposta seja recebida após todas as tentativas, a requisição é registrada como perdida definitivamente.

## Número de sequência

Cada requisição possui um identificador:

```text
CALC:7:10:+:20
```

O servidor retorna o mesmo número:

```text
RESULT:7:30
```

Esse identificador permite associar uma resposta à requisição correspondente.

Em caso de retransmissão, o número de sequência não muda.

Por exemplo:

```text
Tentativa 1:
CALC:7:10:+:20

Tentativa 2:
CALC:7:10:+:20
```

As duas transmissões representam a mesma requisição lógica.

## Métricas

Para cada execução são coletadas as seguintes métricas:

* tempo total da sequência;
* RTT médio;
* RTT máximo;
* número de retransmissões no UDP;
* número de requisições perdidas definitivamente.

## Experimentos e Resultados (Parte 3)

Foram executados 4 cenários experimentais com $N = 20$ requisições cada: 3 execuções com o protocolo UDP variando a taxa de perda simulada em 0%, 10% e 30%, e 1 execução com o protocolo TCP.

### Tabela Comparativa de Resultados

| Protocolo | Taxa de Perda | Tempo Total | RTT Médio (ms) | RTT Máximo (ms) | Retransmissões | Requisições Perdidas |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **UDP** | 0% | 0.005 s (5.451 ms) | 0.225 ms | 0.573 ms | 0 | 0 |
| **UDP** | 10% | 1.508 s (1508.321 ms) | 0.220 ms | 0.869 ms | 3 | 0 |
| **UDP** | 30% | 2.515 s (2514.947 ms) | 0.360 ms | 1.067 ms | 5 | 0 |
| **TCP** | N/A | 0.003 s (2.595 ms) | 0.099 ms | 0.157 ms | N/A (0 na aplicação) | 0 |

### Grade Específica das Execuções UDP

| Taxa de Perda | Tempo total (s) | RTT médio (ms) | Retransmissões | Perdidas definitivamente |
| :---: | :---: | :---: | :---: | :---: |
| **0%** | 0.005 s | 0.225 ms | 0 | 0 |
| **10%** | 1.508 s | 0.220 ms | 3 | 0 |
| **30%** | 2.515 s | 0.360 ms | 5 | 0 |

---

### Detalhamento das Execuções

#### 1. UDP — 0% de perda simulada (`--loss-rate 0.0`)
* **Datagramas enviados:** 20 (nenhum descarte)
* **Retransmissões:** 0
* **Requisições perdidas definitivamente:** 0
* **Tempo total:** 5.451 ms
* **RTT médio:** 0.225 ms
* **RTT máximo:** 0.573 ms
* **Observação:** Sem perdas induzidas, todas as 20 requisições foram respondidas na primeira tentativa. O tempo total foi dominado apenas pelo processamento e latência local da interface de rede (loopback).

#### 2. UDP — 10% de perda simulada (`--loss-rate 0.1`)
* **Datagramas enviados:** 23
* **Retransmissões:** 3
  * Requisição #16: 2 timeouts consecutivos, respondida na 3ª tentativa.
  * Requisição #19: 1 timeout, respondida na 2ª tentativa.
* **Requisições perdidas definitivamente:** 0
* **Tempo total:** 1508.321 ms
* **RTT médio:** 0.220 ms
* **RTT máximo:** 0.869 ms
* **Observação:** Os 3 timeouts de 500 ms cada adicionaram aproximadamente 1,5 segundos ao tempo total da sequência ($3 \times 500\text{ ms} + \text{RTTs} \approx 1508\text{ ms}$). O mecanismo de retransmissão garantiu a entrega de 100% das requisições.

#### 3. UDP — 30% de perda simulada (`--loss-rate 0.3`)
* **Datagramas enviados:** 25
* **Retransmissões:** 5
  * Requisições #0, #2 e #15: sofreram 1 timeout e foram atendidas na 2ª tentativa.
  * Requisição #17: sofreu 2 timeouts consecutivos e foi atendida na 3ª tentativa.
* **Requisições perdidas definitivamente:** 0
* **Tempo total:** 2514.947 ms
* **RTT médio:** 0.360 ms
* **RTT máximo:** 1.067 ms
* **Observação:** Houve um aumento proporcional nas retransmissões (5 no total). Com cada retransmissão aguardando o timeout de 500 ms, o tempo total subiu para ~2,51 segundos ($5 \times 500\text{ ms} + \text{RTTs} \approx 2515\text{ ms}$). Nenhuma requisição atingiu o limite de 5 tentativas descartadas, portanto não houve perdas definitivas.

#### 4. TCP — Conexão padrão
* **Requisições enviadas:** 20
* **Retransmissões (aplicação):** 0 (controle transparente na camada de transporte)
* **Requisições perdidas:** 0
* **Tempo total:** 2.595 ms
* **RTT médio:** 0.099 ms
* **RTT máximo:** 0.157 ms
* **Observação:** No TCP, a conexão pré-estabelecida permitiu um fluxo contínuo e extremamente rápido em loopback. O RTT médio foi de apenas 0.099 ms, sem sobrecarga de timeouts no nível da aplicação.

---

### Análise dos Resultados

1. **Impacto do Timeout no UDP:**
   * No UDP, o RTT medido para mensagens que obtiveram resposta direta permaneceu na faixa de frações de milissegundo (0.2–0.4 ms). No entanto, o **tempo total da sequência** cresce em degraus de 500 ms para cada pacote perdido, já que o cliente precisa esgotar a janela de timeout antes de retransmitir.
   * Isso demonstra que a escolha do valor de timeout em protocolos do tipo Stop-and-Wait é crítica: timeouts conservadores evitam retransmissões espúrias, mas penalizam fortemente a latência total quando há perdas.

2. **Eficácia da Retransmissão da Aplicação:**
   * Mesmo sob taxa de perda de 30%, o limite de 5 tentativas (`MAX_ATTEMPTS = 5`) foi suficiente para garantir que nenhuma requisição fosse perdida em definitivo ($0$ perdas definitivas em todas as rodadas). A probabilidade de 5 perdas seguidas com 30% de perda é $(0.3)^5 = 0.00243$ (menos de 0,25%).

3. **Comparação TCP vs UDP:**
   * Em ambiente local (loopback) sem perdas, o TCP apresentou RTT médio inferior (0.099 ms vs 0.225 ms) e menor variação de latência (RTT máximo de 0.157 ms).
   * No TCP, as garantias de entrega, ordenação e integridade são providas diretamente pelo kernel do sistema operacional, desonerando o código da aplicação de gerenciar timeouts e números de sequência para recuperação de pacotes perdidos. No UDP, a aplicação teve de construir esse mecanismo manualmente.

## Parte 4 — Implementação com Protocol Buffers (protobuf)

Nesta parte, a calculadora remota foi reimplementada utilizando **Protocol Buffers (protobuf)** como formato binário de serialização, substituindo o protocolo textual das partes anteriores.

### 1. Especificação do Protocolo (`proto/calculator.proto`)

```protobuf
syntax = "proto3";

package calculator;

enum Operation {
  ADD = 0;
  SUB = 1;
  MUL = 2;
  DIV = 3;
}

message CalcRequest {
  uint32 sequence = 1;
  double operand1 = 2;
  Operation op = 3;
  double operand2 = 4;
}

message CalcResponse {
  uint32 sequence = 1;
  oneof result_or_error {
    double result = 2;
    string error = 3;
  }
}
```

### 2. Compilação e Geração de Código

Utilizou-se a biblioteca `prost` e o crate `prost-build` em conjunto com o compilador oficial `protoc 36.2`. A compilação é automática durante o `cargo build`, orquestrada pelo arquivo [`build.rs`](build.rs):

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::compile_protos(&["proto/calculator.proto"], &["proto/"])?;
    Ok(())
}
```

### 3. Enquadramento sobre TCP (*Length-Prefixed Framing*)

Diferente do protocolo textual (onde as mensagens eram delimitadas por `\n`), os dados binários do Protobuf podem conter qualquer sequência de bytes. Para delimitar mensagens sobre o stream TCP de forma robusta e padronizada, adotou-se o **Length-Prefixed Framing**:

* **Envio:** O remetente envia 4 bytes inteiros (*big-endian* `u32`) indicando o comprimento exato $L$ do payload serializado, seguido imediatamente pelos $L$ bytes do Protobuf.
* **Recepção:** O receptor executa `read_exact` de 4 bytes para obter $L$, aloca um buffer de tamanho $L$, e realiza um segundo `read_exact` dos $L$ bytes antes de chamar `Message::decode`.

### 4. Executando

Servidor Protobuf:

```bash
cargo run --bin server-proto
```

Cliente Protobuf:

```bash
cargo run --bin client-proto
```

### 5. Comparação: Protocolo Textual (Parte 2) vs Protocolo Protobuf (Parte 4)

Ambos os clientes foram executados transmitindo $N = 20$ requisições matemáticas equivalentes pela mesma interface de rede (*loopback*):

#### Tabela Comparativa de Tamanho das Mensagens (em bytes)

| Métrica | Protocolo Textual (Parte 2) | Protocolo Protobuf (Parte 4) | Variação |
| :--- | :---: | :---: | :---: |
| **Tamanho médio por requisição** | 15.05 bytes | 21.40 bytes | +42.2% |
| **Tamanho médio por resposta** | 13.10 bytes | 10.90 bytes | **-16.8% (mais compacto)** |
| **Tamanho médio global por mensagem** | 14.07 bytes | 16.15 bytes | +14.8% |
| **Total trafegado (apenas payload)** | 563 bytes | 646 bytes | +14.7% |
| **Total trafegado (com framing TCP)** | 563 bytes (usa `\n`) | 806 bytes (+4B prefixo) | +43.2% |

#### Tabela Comparativa de Desempenho e RTT

| Métrica | TCP Textual | TCP Protobuf |
| :--- | :---: | :---: |
| **Tempo total da sequência ($N = 20$)** | 2.579 ms | 3.370 ms |
| **RTT médio por requisição** | 0.096 ms | 0.118 ms |
| **RTT máximo** | 0.154 ms | 0.291 ms |

---

### 6. Análise das Mensagens e Serialização

1. **Por que a requisição do Protobuf foi maior para números pequenos?**
   * No protocolo textual (`CALC:0:10:+:1\n`), números como `10` e `1` ocupam apenas 1 ou 2 caracteres ASCII (1 ou 2 bytes).
   * No Protobuf, `operand1` e `operand2` são tipados como `double` (ponto flutuante IEEE 754 de 64 bits), consumindo rigorosamente 8 bytes cada (16 bytes apenas para os dois operandos), mais as tags de campo e o enum da operação.
   * **Cenário de números reais/grandes:** Se os números fossem valores com muitas casas decimais (ex.: `123456.7891011`), o formato textual explodiria para mais de 35 bytes por requisição, enquanto o Protobuf manteria exatamente os mesmos 21 bytes.

2. **Por que a resposta do Protobuf foi mais compacta?**
   * As respostas textuais em caso de operações com dízimas ou múltiplos dígitos (ex.: `RESULT:15:1.5625\n` com 17 bytes ou `RESULT:11:1.75\n` com 15 bytes) gastam muitos caracteres para representar os números e o prefixo `RESULT:n:`.
   * No Protobuf, o `CalcResponse` codifica o número de sequência via Varint e o resultado como `double` ou `string` em um `oneof`, resultando em apenas 11 bytes na maioria dos casos (e 9 bytes para zero).

3. **Ganhos Arquiteturais do Protobuf:**
   * **Tipagem forte e segurança de tipos:** Sem risco de bugs causados por separação de strings (`split(':')`) ou divergências de localização (`.` vs `,` em números com ponto flutuante).
   * **Parsing binário de alto desempenho:** Decodificação direta em estruturas de memória sem parsing léxico de texto.
   * **Evolução de esquema e compatibilidade:** É possível adicionar novos campos às mensagens no futuro sem quebrar clientes ou servidores antigos.

### 7. Tratamento de Erros

O sistema implementa tratamento defensivo de erros em ambas as pontas:
* **Conexão recusada:** Se o servidor estiver indisponível, o cliente captura o erro do socket nativo (`io::ErrorKind::ConnectionRefused`) e finaliza com mensagem amigável sem sofrer *panic*.
* **Divisão por zero:** Tratada explicitamente pelo servidor, que retorna uma mensagem estruturada no campo `oneof result_or_error`:
  ```text
  --- Teste de Tratamento de Erro (Divisão por Zero) ---
  Servidor retornou erro com sucesso: 'divisao por zero'
  ```
* **Operação inválida:** Mapeada para retorno de erro caso seja enviado um enum não reconhecido.
* **Desconexão graciosa:** O servidor identifica o fim de stream (`UnexpectedEof`) do cliente e libera a thread correspondente sem travar.

## TCP vs UDP

| Característica                            | TCP                 | UDP                     |
| ----------------------------------------- | ------------------- | ----------------------- |
| Orientado a conexão                       | Sim                 | Não                     |
| Confiabilidade                            | Sim                 | Não                     |
| Entrega ordenada                          | Sim                 | Não                     |
| Retransmissão automática                  | Sim                 | Não                     |
| Preserva mensagens                        | Não, utiliza stream | Sim, utiliza datagramas |
| Controle de congestionamento              | Sim                 | Não embutido            |
| Retransmissão implementada pela aplicação | Não                 | Neste projeto, sim      |

## Tecnologias

* Rust (edição 2024)
* `std::net::TcpListener` e `std::net::TcpStream` (Sockets nativos TCP)
* `std::net::UdpSocket` (Sockets nativos UDP)
* `std::thread` (Concorrência por conexão)
* crate `prost` e `prost-build` (Protocol Buffers em Rust)
* Compilador oficial `protoc 36.2`
* crate `rand` (Geração de números aleatórios e simulação de perda)

## Dependências

As dependências são gerenciadas pelo Cargo:

```toml
[dependencies]
rand = "0.10.3"
prost = "0.13"
prost-types = "0.13"

[build-dependencies]
prost-build = "0.13"
```

## Objetivo do experimento

A implementação permite observar, na prática, as principais diferenças arquiteturais de comunicação em sistemas distribuídos:

1. **Camada de Transporte (TCP vs UDP):** No TCP, a aplicação usufrui de um canal confiável fornecido pela camada de transporte. No UDP, a aplicação recebe apenas entrega de datagramas de melhor esforço, exigindo que confiabilidade, números de sequência, timeouts e retransmissões sejam geridos na camada de aplicação.
2. **Serialização (Texto vs Protocol Buffers):** O protocolo textual oferece legibilidade humana imediata ao custo de parsing manual de strings e variação no tamanho das mensagens. O Protocol Buffers oferece serialização binária eficiente, esquema formal com tipagem estática e enquadramento determinístico sobre streams TCP.

