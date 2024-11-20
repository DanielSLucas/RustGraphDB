# GraphDB

Banco de dados orientado a grafos.

## Objetivo

Desenvolver um banco de dados orientado a grafos em rust, aplicando conceitos de programação paralela e grafos.

## Requisitos/planejamento

- Linguagem a ser utilizada:

  **Rust** -> foi escolhida por ser uma liguagem mais baixo nível, altamente eficiente, e prioridade em memory safety.

- Estrutura de projeto:
  
  - `Server` -> Responsável por realizar todas as operações com os grafos, desde as mais básicas como CRUD, bem como algortmos de grafos mais complexos, além de persistir (arquivos .gph) os dados em disco e oferecer uma interface de interação (API)
    - api -> disponibiliza uma camada rest para interagir com o banco
    - graph -> definição das estruturas de dados, como o próprio grafo, nós e arestas. E também algoritmos de busca.
    - query -> responsável por traduzir as consulta em Cypher, para serem executadas pelo banco.
    - storage -> responsável por manejar e persistir os dados tanto em memória como em disco.
    - utils -> funções utilitárias para serem usadas por todo código (e.g. logs)
  - Cli -> Responsável por disponibilizar uma interface de linha de comando para manipular o banco.
  - Test -> Responsável por testar as funcionalidades do banco, e realizar testes de desempenho (benchmark)

## Desenvolvimento

### graph
Iniciamos o desenvolvimento pelo módulo de grafos. Os grafos em si são compostos por seu nome e um hashmap para os nós e outro para as arestas.

Os nós por sua vez tem id, título e propriedades.
As arestas tem id, titulo, o id do nó de origem e destino e propriedades.

inicialmente os id eram números inteiros informados manualmente, posteriomente implementamos um gerador de Ids usando o formato `AtomicInt` do rust, que é um tipo de inteiro que pode ser usado de maneira segura entre threads, evitando a duplicação de ids.

Os algoritmos de busca implementados inicialmente foram o depth first search (dfs) e breadth first search (bfs) de maneiro sigle threaded (sequencial). 

Posteriormente os mesmos algoritmos foram implementados usando multi-threading, fazendo o `spawn` de novas threads para acelerar a busca a depender do tamanho do grafo. 

Afim de adicionar um algoritmo de menor caminho, implementamos o algoritmos de dijkstra, adaptado afim de podermos executá-lo usando propriedades das arestas.

### api

Afim de disponibilizar as funcionalidades do banco de maneira simples e acessível, criamos uma api rest. Que interage com o banco da dados através dos seus endpoints, passando a requisição para os handlers que por sua vez chama o seviço reponsável por lidar com as operações e passa-las para camada de persistencia.

### query

Com o Objetivo de oferecer mais flexibilidade na manipulação do banco, foi implementado a funcionalidade de query, junto ao API REST, que permite ao usuário criar consultas mais complexas ao banco de dados utilizando uma linguagem similar ao Cypher.

### storage

O módulo de storage foi o que mais passou por alterações. Inicialmente os grafos eram só armazenados em memória, mas isso significaria que, uma vez que o banco fosse desligado, os dados se perderiam.

Para persistir os dados independentemente do banco estar executando ou não, decidimos persistir os dados em um json, afim de ser um arquivo fácil de ler por humanos.

Porém com essa abordagem passamos por algusn problemas. Devido a estrutura de um arquivo json, para que houvesse a escrita de novos registros era necessários ler o arquivo inteiro, adicicionar o novo registro, e reeescrever o arquivo com as informações atualizadas. Isso tomava muito tempo, e quando várias leitura e escritas aconteciam ao mesmo tempo ocorriam deadlocks e por fim nada era escrito no arquivo.

Para lidar com isso, decidimos, provisoriamente salvar as alterações em batch, no momento em que o servidor fosse finalizado, através de um gracefull shutdown. Porém com isso outro problema surge, pois se o processo do banco fosse terminado de maneira inesperada, sem que houvesse tempo para salvar os dados, todos os dados se perdiam.

Afim de resolver esses dois problemas, mudamos o formato dos arquivos para csv, pois assim a adição de novos registros se daria através de um simples append, mas que por sua vez poderia causar um novo problema na leitura, já que nós e areas iriam se misturar. Para resolver isso um grafo era armazenado em uma pasta com seu nome e dois arquivos csv de nós e arestas.

Com essa abordagem poderiamos salvar as alterações no grafo após cada operação, porém o problema de deadlocks ainda permanecia. Ao receber várias operações em um mesmo grafo, todos tentavam ler e escrever no mesmo recurso em memória o que levava a um travamento (deadlock).

Para resolver isso foi necessário usar de recursos da linguágem rust, afim de garantir o acesso seguro aos endereços de memória, e também o uso de travas (locks), que permitissem a leitura por vários recursos mas a escrita somente por um único.

Porém a persistencia dos dasdos em csv tinha um problema, Quando registros tinham que ser atualizados ou deletados, isso significaria que o arquivo todo teria que ser lido e reescrito, tornando muito custoso em termos de perfomance.

Então surgimos com uma nova abordagem, as escritas em disco seriam feitas por um thread separada, assim que a thread principal recebesse alguma operação que precisasse ser persistida, uma mensagem é enviada para essa outra thread que lida com as escritas em disco.

Já as escritas em disco passaram as ser feitas em um novo formato. Criamos um arquivo binário, arbitrariamente nomeado com a extensão .gph, pois seria um arquivo para cada grafo.

Esse arquivo teria a seguinte estrutura, um cabeçalho de tamanho fixo com os metadados do grafo, seguido de um bloco de nós e um bloco de arestas, tendo os nós e arestas um tamanho padrão também fixo.

Dessa maneira, com o cabeçalho, nós e arestas tendo tamanos fixos, tornasse possível navegar no arquivo em blocos, movendo-se de maneira mais eficiente e alterando somente o que precisa ser alterado.

Mas o bloco de nós e arestas estarem no mesmo arquivo gerou um novo problema, a escrita de novos nós iria requerer que todos as arestas abaixo dele fossem movidas para baixo. Porém visto que a ordenação dos nós e arestas não é tão relevante para grafos, decidimos que apenas a primeira aresta seria movida para o fim do arquivo, assim evitando de mover todas elas.

### Cli

Afim de disponibilizar mais de uma maneira de interagir com o banco, criamos uma interface de linha de comando. A qual permite ter acesso as funcionalidades do banco através do terminal.

Inicialmente essa CLI estava conectada diretamente no banco, sendo possível apenas iniciar o banco no modo CLI ou então no modo API rest. Mas posteriomente separamos a CLI em um novo pacote, que consome os endpoints disponibilizados pela api rest.

### Test

Como todo projeto existe a necessidade de validar se aquilo que está sendo projetado está tento um resultado satisfatório, assim realizamos a criação do modulo de `Test` com o objetivo de validar tanto se o `server` está funcionando corretamente como também se o servidor está tendo um bom desempenho considerando a quantidade de dados enviados para serem manipulados, assim dentro do `Test` foi criado 3 cenários de testes:
Cenário 1 ->  Envio de vários dados de forma separados por `request`, nesse cenário realizamos a criação de vários nós e arestas, enviando 1 a 1 para o servidor usando `multi-threads` assim, conseguimos verificar se o `server` está conseguindo lidar com vários `request`s de uma só vez e se isso está influenciando na escrita dos nós e arestas.
Cenário 2 -> Envio de vários dados em um único `request`, nesse cenário foi envio um único `request` com todos os nós que devem ser criado e depois um `request` com todas as arestas que devem ser criadas, com esse cenário é possível verificar qual o impacto para o `server` de receber uma serie de `request`s e precisar processar e responder todos eles.
Cenário 3 -> Busca de vários caminhos usando algum dos 3 algoritmos de busca implementado, nesse cenário é feito um `request` passando para o `server` dois nós e algum dos 3 algoritmos de busca, e o `server` deve retornar o caminho com base no algoritmo de busca passado, todo esse processo foi utilizando `multi-threads` para também emular uma situação real do banco recebendo vários pedidos de buscas ao mesmo tempo, com esse cenário foi possível realizar a analise de como o algoritmo de busca consome do banco de dados tempo de processamento. 

## Resultados

Por fim como resultado obtivemos um protótipo de um banco de dados orientados a grafos.

-> duas interfaces de interação possiveis, cli e api. Permitindo a manipulação e consulta de dados em grafos.
-> persistencia de dados em um arquivo proprietário `.gph`. 
-> Desemepenho satisfatório de escrita conforme o resultado dos testes:
    - Cenário 1: 158 nós e 5000 arestas em 2.62800 segundos
    - Cenário 2: 158 nós e 5000 arestas em 384.85 milisegundos
    - Cenário 3: 1000 buscas em 8.76 segundos

## Conclusão

CONCLUAM!!!!!

## Referências

Pesquisamos e consultamos essas caras aqui:

- ChatGPT
- Claude
- Reddit
- StackOverflow