# MiniC

MiniC é uma linguagem pequena, parecida com C, escrita em Rust. Um programa é
só uma lista de funções, e a execução começa em `void main()`. O compilador
lê o arquivo, monta a árvore sintática, confere os tipos e, se passar, roda
o programa num interpretador que caminha por essa árvore. Dá também para
pedir o código de três endereços (TAC) da função `main`, na parte da
linguagem que o gerador já traduz.

O repositório é um fork de [rbonifacio/MiniC](https://github.com/rbonifacio/MiniC).

## O que o repositório diz sobre a disciplina

Não achei, no código nem no histórico, o código da disciplina nem o período
letivo. Não tem "IF688", "Compiladores" como nome de cadeira, nem "2026.1".
O que dá para apontar:

- Os commits de Ian Melo estão assinados com `imm2@cin.ufpe.br`.
- O upstream é o repositório de Rodrigo Bonifácio (`rbonifacio`).
- `docs/09-projects01.md` sorteia o Projeto 2, o laço `for`, para o grupo de
  Ian Medeiros Melo, Victor Mendonça Aguiar, Rafael Alves de Azevedo Silva,
  João Victor Fellows Rabelo e Guilherme Montenegro de Albuquerque.
- O sorteio está reproduzido em `scripts/alocacao-projetos.py`, com
  `random.seed(20260407)`.
- O primeiro commit é de 18/02/2026. O `for` entra em 20 e 21/04/2026. A
  geração de TAC vai de 29/06/2026 a 01/07/2026, data do último commit em
  `main` antes deste README.

Os textos em `docs/` seguem em inglês. Vieram do repositório original e
entram em mais detalhe na linguagem, no parser, no type checker, no
interpretador e nos testes. O enunciado dos projetos de extensão está em
`docs/09-projects01.md`. Os slides de geração de código estão em
`docs/tac_rules.tex` (título: "Code Generation", "From Annotated AST to
Three Address Code").

## Como um arquivo chega a rodar

Não existe um lexer separado. O parser em `src/parser/` usa a biblioteca
[nom](https://docs.rs/nom) (versão 7.1.3, travada no `Cargo.lock`) e lê o
texto direto. A função de entrada é `program`, em `src/parser/program.rs`.
Ela devolve uma AST em que o campo `ty` de cada nó ainda é `()`: o parser
não calcula tipo.

O type checker (`semantic::type_check`, em `src/semantic/`) anda nessa
árvore. Ele exige `void main()` sem parâmetros, variável declarada antes do
uso, chamada com a aridade certa, `return` compatível com o tipo da função,
e condição booleana em `if`, `while` e `for`. Se aceitar, devolve a mesma
árvore com um `Type` em cada nó. O interpretador só recebe essa versão
checada. O tipo do Rust não deixa executar a AST crua do parser.

O interpretador (`src/interpreter/`) é um tree walker. Cada expressão vira
um valor, cada comando é executado, e um `return` sobe até quem chamou a
função. Função do usuário e função da biblioteca padrão ficam no mesmo
ambiente das variáveis. A biblioteca, em `src/stdlib/`, registra `print`,
`readInt`, `readFloat`, `readString`, `sqrt` e `pow`.

`mini_c --tac` chama `translate_program` (`src/codegen/tac_code_gen.rs`) e
imprime o TAC da `main`. Hoje isso cobre declaração, atribuição a variável,
chamada, `if` com `else`, `for`, e algumas expressões: literal, nome, `+`,
`!`, `and`, `or` e condição relacional. `while`, `return`, `if` sem `else`,
atribuição em posição de array e o resto das operações caem num `todo!()`.
O processo aborta com exit 101. O programa pode ser válido; o gerador é que
ainda não traduz aquele trecho.

O parser de programa usa `many0` e para no primeiro pedaço que não é uma
declaração de função, sem erro de sintaxe. O resto do arquivo é ignorado.
Se o `main` não chegou a ser lido, o type checker diz
`program must have a main function`. É o que `tests/cli/check.test` espera
para `tests/fixtures/invalid_syntax.minic`.

Comando simples termina com `;`. O corpo de `if`, `while` e `for` tem que
ser um bloco entre chaves. O `for` é o de três partes, e qualquer uma delas
pode faltar:

```c
for (int i = 0; i < 10; i = i + 1) {
  sum = sum + i;
}
```

## Como rodar

Conferido em Linux no dia 30/10/2026, com o stable do Rust de então:

```text
rustc 1.99.1 (58cf8f9c3 2026-10-13)
cargo 1.99.1 (5f94df478 2026-08-27)
```

Na primeira compilação o Cargo baixa da rede o que o `Cargo.lock` fixa:
`nom` 7.1.3, `memchr` 2.8.0 e `minimal-lexical` 0.2.1. Da raiz do
repositório:

```bash
cargo build
cargo test
```

`cargo test` passou 205 testes. Um doctest em `src/interpreter/value.rs`
fica de fora de propósito (`ignore`): ele só mostra a assinatura de um
ponteiro de função, não é um teste executável.

Os arquivos `tests/cli/*.test` são do shelltestrunner, um programa em
Haskell. Ele não vem com o Rust, e aqui não estava instalado. `cargo test`
não lê esses arquivos. Os mesmos casos estão reproduzidos abaixo, chamando
o binário.

O pacote se chama `mini_c`. O executável é `target/debug/mini_c`.

```bash
./target/debug/mini_c --run tests/fixtures/interpreter_hello.minic
./target/debug/mini_c --run examples/fatorial.minic
./target/debug/mini_c --run examples/soma.minic
./target/debug/mini_c --check tests/fixtures/interpreter_hello.minic
./target/debug/mini_c --check tests/fixtures/cli_type_mismatch.minic
./target/debug/mini_c --tac tests/fixtures/tac_simple.minic
./target/debug/mini_c --tac examples/if_else.minic
cargo run --quiet --example dump_ast -- tests/fixtures/tac_simple.minic
```

`dump_ast` não faz parte do compilador. É um exemplo que imprime a AST do
parser com o formato `Debug` do Rust.

Saída do processo: 0 quando deu certo; 1 para erro de uso, de leitura, de
parse, de tipo ou de execução; 101 quando o `--tac` esbarra num `todo!()`.

## Exemplos

Saídas copiadas da execução destes comandos, depois de `cargo build`.

`tests/fixtures/interpreter_hello.minic`:

```c
void main() {
    print(42);
    print(true);
    print("hello");
}
```

```text
$ ./target/debug/mini_c --run tests/fixtures/interpreter_hello.minic
42
true
hello
```

`examples/fatorial.minic` calcula 10!:

```c
int factorial(int n) {
  if n <= 1 { return 1; }
  return n * factorial(n - 1);
}

void main() {
  print(factorial(10));
}
```

```text
$ ./target/debug/mini_c --run examples/fatorial.minic
3628800
```

`examples/soma.minic` soma `0 + 1 + ... + 9` com `for`:

```text
$ ./target/debug/mini_c --run examples/soma.minic
45
```

`tests/fixtures/interpreter_array.minic` troca o elemento do meio de um
array e imprime os três valores:

```text
$ ./target/debug/mini_c --run tests/fixtures/interpreter_array.minic
10
99
30
```

`--check` para no type checker. Programa aceito:

```text
$ ./target/debug/mini_c --check tests/fixtures/interpreter_hello.minic
'tests/fixtures/interpreter_hello.minic' is well-typed.
```

`tests/fixtures/cli_type_mismatch.minic` declara `int x = "hello";`. O
parser aceita. O type checker recusa, na stderr, e o processo sai com 1:

```text
Type error: declaration of x: expected Int, got Str
```

`examples/divzero.minic` é `int x = 1 / 0;`. Isso é inteiro dividido por
zero, então não chega a imprimir `x`. A stderr fica:

```text
RuntimeError: division by zero
```

TAC de `tests/fixtures/tac_simple.minic` (`int x = 1; x = x + 2;`):

```text
$ ./target/debug/mini_c --tac tests/fixtures/tac_simple.minic
main
x = 1
temp1 = x + 2
x = temp1
```

TAC de `examples/if_else.minic`:

```text
main
x = 1
if x > 0 goto Label1:
goto Label2:
Label1:
x = 2
goto Label3:
Label2:
x = 3
Label3:
```

A árvore do parser para o mesmo `tac_simple.minic`, via
`cargo run --quiet --example dump_ast -- tests/fixtures/tac_simple.minic`:

```text
Program {
    functions: [
        FunDecl {
            name: "main",
            params: [],
            return_type: Unit,
            body: StatementD {
                stmt: Block {
                    seq: [
                        StatementD {
                            stmt: Decl {
                                name: "x",
                                ty: Int,
                                init: ExprD {
                                    exp: Literal(
                                        Int(
                                            1,
                                        ),
                                    ),
                                    ty: (),
                                },
                            },
                            ty: (),
                        },
                        StatementD {
                            stmt: Assign {
                                target: ExprD {
                                    exp: Ident(
                                        "x",
                                    ),
                                    ty: (),
                                },
                                value: ExprD {
                                    exp: Add(
                                        ExprD {
                                            exp: Ident(
                                                "x",
                                            ),
                                            ty: (),
                                        },
                                        ExprD {
                                            exp: Literal(
                                                Int(
                                                    2,
                                                ),
                                            ),
                                            ty: (),
                                        },
                                    ),
                                    ty: (),
                                },
                            },
                            ty: (),
                        },
                    ],
                },
                ty: (),
            },
        },
    ],
}
```

O `()` em `ty` é de propósito: essa árvore ainda não passou pelo type checker.

Os arquivos em `docs/prints/` são o terminal, na mesma máquina, rodando estes comandos:

- `01-testes-interpreter.png`: `cargo test --offline --color=never --test interpreter`, com o nome dos 34 testes.
- `02-testes.png`: `cargo test --offline --color=never --quiet`, o resumo de cada suíte (205 testes, 1 doctest ignorado).
- `03-exemplos.png`: hello, fatorial, soma com `for` e o array.
- `04-tipos.png`: `--check` aceito, erro de tipo, divisão por zero e o `invalid_syntax.minic`.
- `05-tac.png`: TAC da atribuição, do `if`/`else` e do `for`.
- `06-ast.png`: a AST de `tac_simple.minic`.

## Onde está cada parte

```text
src/parser/       texto -> AST sem tipos
src/semantic/     type checker
src/interpreter/  execução
src/codegen/      AST checada -> TAC
src/stdlib/       print, leitura e math
src/ir/           nós da AST e do TAC
src/environment/  tabela de símbolos
examples/         programas .minic e o dump_ast
tests/            testes de integração e fixtures
```
