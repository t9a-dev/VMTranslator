# 概要
[コンピュータシステムの理論と実装 第2版](https://www.oreilly.co.jp/books/9784814400874/)8章で学んだ内容に基づいて、個人的に実装した VMTranslator です。
学習目的で作成したコードを公開しています。

# 利用方法
- Rustの[インストール](https://www.rust-lang.org/tools/install)
- [nand2tetris](https://www.nand2tetris.org/software)からNand to Tetris Software packageをダウンロード
- Nand to Tetris Software packageのprojects/8配下の複数`.vm`ファイルを内包するディレクトリを利用する
- `FibonacciElement`ディレクトリを利用する場合の実行例
  - `cargo run -- ./FibonacciElement`