<h1 align="center">
  <br />
  <img src="https://raw.githubusercontent.com/wiki/greymd/teip/img/logo.png" width="208" />
  <h4 align="center">Highly efficient "Masking tape" for Shell</h4>
</h1>
<p align="center">
  <a href="https://github.com/greymd/teip/releases/latest"><img src="https://img.shields.io/github/release/greymd/teip.svg" alt="Latest version" /></a>
  <a href="https://crates.io/crates/teip" alt="crate.io"><img src="https://img.shields.io/crates/v/teip.svg"/></a>
  <a href="https://github.com/greymd/teip/actions?query=workflow%3ATest"><img src="https://github.com/greymd/teip/workflows/Test/badge.svg?branch=master" alt="Test Status" /></a>
  <a href="LICENSE" alt="MIT License"><img src="http://img.shields.io/badge/license-MIT-blue.svg?style=flat" /></a>
</p>

# TL;DR

<p align="center">
  <img src="https://raw.githubusercontent.com/wiki/greymd/teip/img/teip_intro.png" alt="Git Animation for Introduction" width="50%" />
</p>

* Edit 4th and 6th columns in the CSV file

```bash
$ cat file.csv | teip -d, -f 4,6 -- sed 's/./@/g'
```

* Convert timestamps in /var/log/secure to UNIX time

```bash
$ cat /var/log/secure | teip -c 1-15 -- date -f- +%s
```

* Percent-encode bare-minimum range of the file

```bash
$ cat file | teip -og '[^-a-zA-Z0-9@:%._\+~#=/]+' -- php -R 'echo urlencode($argn)."\n";'
```

# Performance enhancement
`teip` allows a command to focus on its own task.

Here is the comparison of processing time to replace approx 761,000 IP addresses with dummy ones in 100 MiB text file.

<p align="center">
  <img src="https://raw.githubusercontent.com/wiki/greymd/teip/benchmark/secure_bench.svg" width="80%" alt="benchmark bar chart" />
</p>

See detail on <a href="https://github.com/greymd/teip/wiki/Benchmark">wiki > Benchmark</a>.

# Features

* Allows any command to "ignore unwanted input" which most commands cannot do
  - The targeted command just handles selected parts of the standard input
  - Unselected parts are bypassed by `teip`
  - Flexible methods for selecting a range (Select like AWK, `cut` command, or a regular expression)

* High performer
  - The targeted command's standard input/output are intercepted by multiple `teip`'s threads asynchronously.
  - If general UNIX commands on your environment can process a few hundred MB files in a few seconds, then `teip` can do the same or better performance.

# Installation

### On macOS (x86_64)

Using [Homebrew](https://brew.sh/)

```bash
$ brew install greymd/tools/teip
```

### With `dpkg` on Ubuntu, Debian, etc (x86_64)

<!-- deb_x86_64_start -->
```bash
$ wget https://git.io/teip-1.2.0.x86_64.deb
$ sudo dpkg -i ./teip*.deb
```
<!-- deb_x86_64_end -->
<!-- deb_x86_64_sha256 -->SHA256: 70a15214a8c1b0a894ae3f83ffcd649a2710d7cb68c660451283f5167c876c80

### With `dnf` on Fedora, CentOS, RHEL, etc (x86_64)

<!-- rpm_x86_64_start -->
```bash
$ sudo dnf install https://git.io/teip-1.2.0.x86_64.rpm
```
<!-- rpm_x86_64_end -->
<!-- rpm_x86_64_sha256 -->SHA256: b8eab16589ff49d6db3b9377e122516217fa0e03ac192a76b4c64a860c096540

### With `yum` on CentOS7, RHEL7, etc (x86_64)

<!-- rpm_x86_64_start -->
```bash
$ sudo yum install https://git.io/teip-1.2.0.x86_64.rpm
```
<!-- rpm_x86_64_end -->
<!-- rpm_x86_64_sha256 -->SHA256: b8eab16589ff49d6db3b9377e122516217fa0e03ac192a76b4c64a860c096540

### With Docker

```bash
$ docker build -t teip .
$ echo "100 200 300 400" | docker run --rm -i teip -f 3 -- sed 's/./@/g'
```

### On other UNIX or other architectures (i686, ARM, etc..)

Pre-built binary is not prepared for now.
Build with `cargo`, then make sure `libclang` shared library is on your environment.

```bash
### Example for Ubuntu
$ sudo apt install cargo clang
$ cargo install teip
```

```bash
### Example for RHEL
$ sudo dnf install cargo clang
$ cargo install teip
```

### For Windows

Unfortunately, `teip` does not work on non-UNIX environment due to technical reason.

### From source

With Rust's package manager cargo, you can install `teip` via:

cargo install teip

# Usage

```
Usage:
  teip -g <pattern> [-oGsvz] [--] [<command>...]
  teip -f <list> [-d <delimiter> | -D <pattern>] [-svz] [--] [<command>...]
  teip -c <list> [-svz] [--] [<command>...]
  teip -l <list> [-svz] [--] [<command>...]
  teip --help | --version

Options:
  --help          Display this help and exit
  --version       Show version and exit
  -g <pattern>    Select lines that match the regular expression <pattern>
  -o              -g selects only matched parts.
  -G              -g adopts Oniguruma regular expressions
  -f <list>       Select only these white-space separated fields
  -d <delimiter>  Use <delimiter> for field delimiter of -f
  -D <pattern>    Use regular expression <pattern> for field delimiter of -f
  -c <list>       Select only these characters
  -l <list>       Select only these lines
  -s              Execute command for each selected part
  -v              Invert the sense of selecting
  -z              Line delimiter is NUL instead of newline
```

## Getting Started

Try this at first.

```bash
$ echo "100 200 300 400" | teip -f 3
```

The result is almost the same as the input but "300" is highlighted and surrounded by `[...]`.
Because `-f 3` selects the 3rd field of space-separated input.

```bash
100 200 [300] 400
```

Next, put the `sed` and its arguments at the end.

```bash
$ echo "100 200 300 400" | teip -f 3 sed 's/./@/g'
```

The result is as below.
Highlight and `[...]` is gone then.

```
100 200 @@@ 400
```

As you can see, `teip` passes only highlighted part to the `sed` and replaces it with the result of the `sed`.

Off-course, any command whatever you like can be specified.
It is called the **targeted command** in this article.

Let's try the `cut` as the targeted command to extract the first character only.

```bash
$ echo "100 200 300 400" | teip -f 3 cut -c 1
teip: Invalid arguments.
```

Oops? Why is it failed?

This is because the `cut` uses the `-c` option.
The option of the same name is also provided by `teip`, which is confusing.

When entering a targeted command with `teip`, it is better to enter it after `--`.
Then, `teip` interprets the arguments after `--` as the targeted command and its argument.

```bash
$ echo "100 200 300 400" | teip -f 3 -- cut -c 1
100 200 3 400
```

Great, the first character `3` is extracted from `300`!

Although `--` is not always necessary, it is always better to be used.
So, `--` is used in all the examples from here.

Now let's double this number with the `awk`.
The command looks like the following (Note that the variable to be doubled is not `$3`).

```bash
$ echo "100 200 300 400" | teip -f 3 -- awk '{print $1*2}'
100 200 600 400
```

OK, the result went from 300 to 600.

Now, let's change `-f 3` to `-f 3,4` and run it.

```bash
$ echo "100 200 300 400" | teip -f 3,4 -- awk '{print $1*2}'
100 200 600 800
```

The numbers in the 3rd and 4th were doubled!

As some of you may have noticed, the argument of `-f` is compatible with the __LIST__ of `cut`.

Let's see how it works with `cut --help`.

```bash
$ echo "100 200 300 400" | teip -f -3 -- sed 's/./@/g'
@@@ @@@ @@@ 400

$ echo "100 200 300 400" | teip -f 2-4 -- sed 's/./@/g'
100 @@@ @@@ @@@

$ echo "100 200 300 400" | teip -f 1- -- sed 's/./@/g'
@@@ @@@ @@@ @@@
```

## Select range by character

The `-c` option allows you to select a range by character-base.
The below example is selecting 1st, 3rd, 5th, 7th characters and apply the `sed` command to them.

```bash
$ echo ABCDEFG | teip -c 1,3,5,7
[A]B[C]D[E]F[G]

$ echo ABCDEFG | teip -c 1,3,5,7 -- sed 's/./@/'
@B@D@F@
```

As same as `-f`, `-c`'s argument is compatible with `cut`'s __LIST__.

## Processing delimited text like CSV, TSV

The `-f` option recognizes delimited fields [like `awk`](https://www.gnu.org/software/gawk/manual/html_node/Regexp-Field-Splitting.html) by default.

The continuous white spaces (all forms of whitespace categorized by [Unicode](https://www.unicode.org/Public/UCD/latest/ucd/PropList.txt)) is interpreted as a single delimiter.

```bash
$ printf "A       B \t\t\t\   C \t D" | teip -f 3 -- sed s/./@@@@/
A       B                       @@@@   C         D
```

This behavior might be inconvenient for the processing of CSV and TSV.

However, the `-d` option in conjunction with the `-f` can be used to specify a delimiter.
Now you can process the CSV file like this.

```bash
$ echo "100,200,300,400" | teip -f 3 -d , -- sed 's/./@/g'
100,200,@@@,400
```

In order to process TSV, the TAB character need to be typed.
If you are using Bash, type `$'\t'` which is one of [ANSI-C Quoting](https://www.gnu.org/software/bash/manual/html_node/ANSI_002dC-Quoting.html).

```bash
$ printf "100\t200\t300\t400\n" | teip -f 3 -d $'\t' -- sed 's/./@/g'
100     200     @@@     400
```

`teip` also provides `-D` option to specify an extended regular expression as the delimiter.
This is useful when you want to ignore consecutive delimiters, or when there are multiple types of delimiters.

```bash
$ echo 'A,,,,,B,,,,C' | teip -f 2 -D ',+'
A,,,,,[B],,,,C
```

```bash
$ echo "1970-01-02 03:04:05" | teip -f 2-5 -D '[-: ]'
1970-[01]-[02] [03]:[04]:05
```

The regular expression of TAB character (`\t`) can also be specified with the `-D` option, but `-d` has slightly better performance.
Regarding available notations of the regular expression, refer to [regular expression of Rust](https://docs.rs/regex/1.3.7/regex/).

## Matching with Regular Expression

You can also select particular lines that match a regular expression with `-g`.

```bash
$ echo -e "ABC1\nEFG2\nHIJ3" | teip -g '[GJ]\d'
ABC1
[EFG2]
[HIJ3]
```

By default, whole the line including the given pattern is selected like the `grep` command.
With `-o` option, only matched parts are selected.

```bash
$ echo -e "ABC1\nEFG2\nHIJ3" | teip -og '[GJ]\d'
ABC1
EF[G2]
HI[J3]
```

Note that `-og` is one of the useful idiom and freuquently used in this manual.

Here is an example of using `\d` which matches numbers.

```bash
$ echo ABC100EFG200 | teip -og '\d+'
ABC[100]EFG[200]

$ echo ABC100EFG200 | teip -og '\d+' -- sed 's/.*/@@@/g'
ABC@@@EFG@@@
```

This feature is quite versatile and can be useful for handling the file that has no fixed form like logs, markdown, etc.

However, you should pay attention to use it.

The below example is almost the same as above one but `\d+` is replaced with `\d`.

```bash
$ echo ABC100EFG200 | teip -og '\d' -- sed 's/.*/@@@/g'
ABC@@@@@@@@@EFG@@@@@@@@@
```

Although the selected characters are the same, the result is different.

It is necessary to know the "Tokenization" of `teip` in order to understand this behavior.

## Tokenization

`teip` divides the standard input into tokens.
A token that does not match the pattern will be displayed on the standard output as it is. On the other hand, the matched token is passed to the standard input of a targeted command.
After that, the matched token is replaced with the result of the targeted command.

In the next example, the standard input is divided into four tokens as follows.

```bash
echo ABC100EFG200 | teip -og '\d+' -- sed 's/.*/@@@/g'
```

```
ABC => Token(1)
100 => Token(2) -- Matched
EFG => Token(3)
200 => Token(4) -- Matched
```

By default, the matched tokens are combined by line breaks and used as the new standard input for the targeted command.
Imagine that `teip` executes the following command in its process.

```bash
$ printf "100\n200\n" | sed 's/.*/@@@/g'
@@@ # => Result of Token(2)
@@@ # => Result of Token(4)
```

(It is not technically accurate but you can now see why `$1` is used not `$3` in one of the examples in "Getting Started")

After that, matched tokens are replaced with each line of result.

```
ABC => Token(1)
@@@ => Token(2) -- Replaced
EFG => Token(3)
@@@ => Token(4) -- Replaced
```

Finally, all the tokens are concatenated and the following result is printed.

```
ABC@@@EFG@@@
```

Practically, the above process is performed asynchronously.
Tokens being printed sequentially as they become available.

Back to the story, the reason why a lot of `@` are printed in the example below is that the input is broken up into many tokens.

```bash
$ echo ABC100EFG200 | teip -og '\d'
ABC[1][0][0]EFG[2][0][0]
```

`teip` recognizes input matched with the entire regular expression as a single token.
`\d` matches a single digit, and it results in many tokens.

```
ABC => Token(1)
1   => Token(2) -- Matched
0   => Token(3) -- Matched
0   => Token(4) -- Matched
EFG => Token(5)
2   => Token(6) -- Matched
0   => Token(7) -- Matched
0   => Token(8) -- Matched
```

Therefore, `sed` loads many newline characters.

```bash
$ printf "1\n0\n0\n2\n0\n0\n" | sed 's/.*/@@@/g'
@@@ # => Result of Token(2)
@@@ # => Result of Token(3)
@@@ # => Result of Token(4)
@@@ # => Result of Token(6)
@@@ # => Result of Token(7)
@@@ # => Result of Token(8)
```

The tokens of the final form are like the following.

```
ABC => Token(1)
@@@ => Token(2) -- Replaced
@@@ => Token(3) -- Replaced
@@@ => Token(4) -- Replaced
EFG => Token(5)
@@@ => Token(6) -- Replaced
@@@ => Token(7) -- Replaced
@@@ => Token(8) -- Replaced
```

And, here is the final result.

```
ABC@@@@@@@@@EFG@@@@@@@@@
```

The concept of tokenization is also used for other options.
For example, if you use `-f` to specify a range of `A-B`, each field will be a separate token.
Also, the field delimiter is always an unmatched token.

```bash
$ echo "AA,BB,CC" | teip -f 2-3 -d,
AA,[BB],[CC]
```

With the `-c` option, adjacent characters are treated as the same token even if they are separated by `,`.

```bash
$ echo "ABCDEFGHI" | teip -c1,2,3,7-9
[ABC]DEF[GHI]
```

## What command can be used?

As explained, `teip` replaces tokens on a row-by-row basis.
Therefore, a targeted command must follow the below rule.

* **A targeted command must print a single line of result for each line of input.**

In the simplest example, the `cat` command always succeeds.
Because the `cat` prints the same number of lines against the input.

```bash
$ echo ABCDEF | teip -og . -- cat
ABCDEF
```

If the above rule is not satisfied, the result will be inconsistent.
For example, `grep` may fail.
Here is an example.

```bash
$ echo ABCDEF | teip -og .
[A][B][C][D][E][F]

$ echo ABCDEF | teip -og . -- grep '[ABC]'
ABC
teip: Output of given command is exhausted

$ echo $?
1
```

`teip` could not get the result corresponding to the token of D, E, and F.
That is why the above example fails.

If an inconsistency occurs, `teip` will exit with the error message.
Also, the exit status will be 1.

## Advanced usage

### Solid mode

If you want to use a command that does not satisfy the condition, **"A targeted command must print a single line of result for each line of input"**, enable "Solid mode" which is available with the `-s` option.

Solid mode spawns the targeted command for each matched token and executes it each time.

```bash
$ echo ABCDEF | teip -s -og . -- grep '[ABC]'
```

In the above example, understand the following commands are executed in `teip`'s procedure.

```bash
$ echo A | grep '[ABC]' # => A
$ echo B | grep '[ABC]' # => B
$ echo C | grep '[ABC]' # => C
$ echo D | grep '[ABC]' # => Empty
$ echo E | grep '[ABC]' # => Empty
$ echo F | grep '[ABC]' # => Empty
```

The empty result is replaced with an empty string.
Therefore, D, E, and F tokens are replaced with empty as expected.

```bash
$ echo ABCDEF | teip -s -og . -- grep '[ABC]'
ABC

$ echo $?
0
```

However, this option is not suitable for processing a large file because it may significantly degrade performance instead of consolidating the results.

### Overlay `teip`s

Any command can be used with `teip`, surprisingly, even if it is **`teip` itself**.

```bash
$ echo "AAA@@@@@AAA@@@@@AAA" | teip -og '@.*@'
AAA[@@@@@AAA@@@@@]AAA

$ echo "AAA@@@@@AAA@@@@@AAA" | teip -og '@.*@' -- teip -og 'A+'
AAA@@@@@[AAA]@@@@@AAA

$ echo "AAA@@@@@AAA@@@@@AAA" | teip -og '@.*@' -- teip -og 'A+' -- tr A _
AAA@@@@@___@@@@@AAA
```

In other words, you can connect the multiple features of `teip` with AND conditions for more complex range selection.
Furthermore, it works asynchronously and in multi-processes, similar to the shell pipeline.
It will hardly degrade performance unless the machine faces the limits of parallelism.

### Oniguruma regular expression

If `-G` option is given together with `-g`, the regular expressin is interpreted as [Oniguruma regular expression](https://github.com/kkos/oniguruma/blob/master/doc/RE). For example, "keep" and "look-ahead" syntax can be used.

```bash
$ echo 'ABC123DEF456' | teip -G -og 'DEF\K\d+'
ABC123DEF[456]

$ echo 'ABC123DEF456' | teip -G -og '\d+(?=D)'
ABC[123]DEF456
```

Those techniques are helpful to reduce the number of "Overlay".

### Empty token

If a blank field exists when the `-f` option is used, the blank is not ignored and treated as an empty token.

```bash
$ echo ',,,' | teip -d , -f 1-
[],[],[],[]
```

Therefore, the following command can work (Note that `*` matches empty as well).

```bash
$ echo ',,,' | teip -f 1- -d, sed 's/.*/@@@/'
@@@,@@@,@@@,@@@
```

In the above example, the `sed` loads four newline characters and prints `@@@` four times.

### Invert match

The `-v` option allows you to invert the selected range.
When the `-f` or `-c` option is used, the complement of the selected field is selected instead.

```bash
$ echo 1 2 3 4 5 | teip -v -f 1,3,5 -- sed 's/./_/'
1 _ 3 _ 5
```

Of course, it can also be used for the `-og` option.

```bash
$ printf 'AAA\n123\nBBB\n' | teip -vr '\d+' -- sed 's/./@/g'
@@@
123
@@@
```

### NUL as line delimiter

If you want to process the data in a more flexible way, the `-z` option may be useful.
This option allows you to use the NUL character (the ASCII NUL character) instead of the newline character.
It behaves like `-z` provided by GNU sed or GNU grep, or `-0` option provided by xargs.

```bash
$ printf '111,\n222,33\n3\0\n444,55\n5,666\n' | teip -z -f3 -d,
111,
222,[33
3]
444,55
5,[666]
```

With this option, the standard input is interpreted per a NUL character rather than per a newline character.
You should also pay attention to that matched tokens are concatenated with the NUL character instead of a newline character in `teip`'s procedure.

In other words, if you use a targeted command that cannot handle NUL characters (and cannot print NUL-separated results), the final result can be unintended.

```bash
$ printf '111,\n222,33\n3\0\n444,55\n5,666\n' | teip -z -f3 -d, -- sed -z 's/.*/@@@/g'
111,
222,@@@
444,55
5,@@@

$ printf '111,\n222,33\n3\0\n444,55\n5,666\n' | teip -z -f3 -d, -- sed 's/.*/@@@/g'
111,
222,@@@
@@@
444,55
5,teip: Output of given command is exhausted
```

Specifying from one line to another is a typical use case for this option.

```bash
$ cat test.html | teip -z -og '<body>.*</body>'
<html>
<head>
  <title>AAA</title>
</head>
[<body>
  <div>AAA</div>
  <div>BBB</div>
  <div>CCC</div>
</body>]
</html>

$ cat test.html | teip -z -og '<body>.*</body>' -- grep -a BBB
<html>
<head>
  <title>AAA</title>
</head>
  <div>BBB</div>
</html>
```

# Environment variables

`teip` refers to the following environment variables.
Add the statement to your default shell's startup file (i.e `.bashrc`, `.zshrc`) to change them as you like.

### `TEIP_HIGHLIGHT`

**DEFAULT VALUE:** `\x1b[36m[\x1b[0m\x1b[01;31m{}\x1b[0m\x1b[36m]\x1b[0m`

The default format for highlighting matched token.
It must include at least one `{}` as a placeholder.

Example:
```
$ export TEIP_HIGHLIGHT="<<<{}>>>"
$ echo ABAB | teip -og A
<<<A>>>B<<<A>>>B

$ export TEIP_HIGHLIGHT=$'\x1b[01;31m{}\x1b[0m'
$ echo ABAB | teip -og A
ABAB  ### Same color as grep
```

[ANSI Escape Sequences](https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797) and [ANSI-C Quoting](https://www.gnu.org/software/bash/manual/html_node/ANSI_002dC-Quoting.html) are helpful to customize this value.

# Background

## Why made it?
See this [post](https://dev.to/greymd/teip-masking-tape-for-shell-is-what-we-needed-5e05).

## Why "teip"?

* [tee](https://en.wikipedia.org/wiki/Tee_%28command%29) + in-place.
* Came from Irish verb "teip" which means "fail" and it can also mean "cut off".
* And it sounds similar to Masking-"tape".

# License

## Modules imported from other repositories

Thank you so much for helpful modules!

* ./src/list/ranges.rs
  - One of the module used in `cut` command of [uutils/coreutils](https://github.com/uutils/coreutils)
  - Original souce codes are distributed under MIT license
  - The license file is on the same directory

## Source code
The scripts are available as open source under the terms of the [MIT License](http://opensource.org/licenses/MIT).

## Logo
<a rel="license" href="http://creativecommons.org/licenses/by-nc/4.0/"><img alt="Creative Commons License" style="border-width:0" src="https://i.creativecommons.org/l/by-nc/4.0/88x31.png" /></a><br />The logo of teip is licensed under a <a rel="license" href="http://creativecommons.org/licenses/by-nc/4.0/">Creative Commons Attribution-NonCommercial 4.0 International License</a>.
