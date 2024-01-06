<!--
Use md2man (https://github.com/sunaku/md2man) to generate the man file like this.
$ md2man-roff man.md > teip.1
-->
TEIP 1 "JAN 2024" "User Commands" ""
=======================================

NAME
----

teip - Masking tape to help commands "do one thing well"

SYNOPSIS
--------

`teip` -g <*pattern*> [-oGsvz] [--] [<*command*>...]

`teip` -f <*list*> [-d <*delimiter*> | -D <*pattern*> | --csv] [-svz] [--] [<*command*>...]

`teip` -c <*list*> [-svz] [--] [<*command*>...]

`teip` -l <*list*> [-svz] [--] [<*command*>...]

`teip` -e <*string*> [-svz] [--] [<*command*>...]

`teip` --help | --version

DESCRIPTION
-----------
Bypassing a partial range of standard input to any command whatever you want

OPTIONS
-------
`--help`
  Prints help information

`-V`, `--version`
  Prints version information

`-g` <*pattern*>
  Act on lines that match the regular expression <*pattern*>

`-o`
  -g acts on only matched parts

`-G`
  -g interprets Oniguruma regular expressions

`-f` <*list*>
  Act on these white-space separated fields

`-d` <*delimiter*>
  Use <*delimiter*> for the field delimiter of -f

`-D` <*pattern*>
  Use regular expression <*pattern*> for the field delimiter of -f

`-c` <*list*>
  Act on these characters

`-e` <*string*>
  Execute <*string*> on another process that will receive identical standard input as the main teip aommane, emitting numbers to be used as line numbers for actioning.

`-l` <*list*>
  Act on these lines

`--csv`
  -f interprets <*list*> as field numbers of a CSV according to RFC 4180, instead of whitespace separated fields

`-s`
  Execute a new command for each actioned chunk

`-I`
  Replace the <*replace-str*> with the actioned chunk in <*command*>, implying -s

`--chomp`
  The command spawned by -s receives the standard input without trailing newlines

`-v`
  Invert the range of actioning

`-z`
  Line delimiter is NUL instead of a newline

-A <*number*>
  Use  together with `-g <pattern>`.
  Alias of `-e 'grep -n -A <number> <pattern>'`

-B <*number*>
  Use together with `-g <pattern>`.
  Alias of `-e 'grep -n -B <number> <pattern>'`

-C <*number*>
  Use  together with `-g <pattern>`.
  Alias of `-e 'grep -n -C <number> <pattern>'`

--sed <*pattern*>
  Alias of `-e 'sed -n "<pattern>="'`
  See also sed(1)

--awk <*pattern*>
  Alias of `-e 'awk "<pattern>{print NR}"'`
  See also awk(1)

### *command*

*command* refers to the command and its arguments that `teip` executes.
Each *command* should output a single line of result for each line of input.
In the simplest example, the `cat(1)` command is always successful,
as `cat` outputs the same number of lines as its input.

```
$ echo ABCDEF | teip -og . -- cat
ABCDEF
```

The `sed(1)` command works well with typical patterns.

```
$ echo ABCDEF | teip -og . -- sed 's/[ADF]/@/'
@BC@E@
```

If the rule is not adhered to, the result may be inconsistent.
For example, `grep(1)` might fail, as shown here.

```
$ echo ABCDEF | teip -og . -- grep '[ABC]'
ABC
teip: Output of given command is exhausted
```

In this case, `teip` could not retrieve results corresponding to D, E, and F, leading to the failure of the example.
If such inconsistency occurs, `teip` will terminate with an error message, and the exit status will be set to 1.

```
$ echo $?
1
```

If no *command* is specified, `teip` displays how the standard input will be divided into chunks.

```
$ echo ABCDEF | teip -og .
[A][B][C][D][E][F]
```

### *list*

*list* is a specification used to define the range of fields or characters.
This notation is compatible with that used in `cut(1)`. For detailed information, please refer to the `cut(1)` manual.
Here are some examples:

Select the 1st, 3rd, and 5th fields.

```
$ echo 1 2 3 4 5 | teip -f 1,3,5 -- sed 's/./@/'
@ 2 @ 4 @
```

Select fields from the 2nd to the 4th.

```
$ echo 1 2 3 4 5 | teip -f 2-4 -- sed 's/./@/'
1 @ @ @ 5
```

Select all fields starting from the 3rd.

```
$ echo 1 2 3 4 5 | teip -f 3- -- sed 's/./@/'
1 2 @ @ @
```

Select all fields up to the 4th.

```
$ echo 1 2 3 4 5 | teip -f -4 -- sed 's/./@/'
@ @ @ @ 5
```

### *pattern*

*pattern* is a regular expression based on the "regex crate" syntax.
For more details, refer to the link in the *SEE ALSO* section.

### The Necessity of **--**

`teip` interprets arguments after `--` as a *command* and its arguments.

Omitting **--** causes failure in this example:

```
$ echo "100 200 300 400" | teip -f 3 cut -c 1
teip: Invalid arguments.
```

This error occurs because `cut` uses the `-c` option, which is also a `teip` option, leading to confusion.

However, with the use of **--**, the command executes successfully:

```
$ echo "100 200 300 400" | teip -f 3 -- cut -c 1
100 200 3 400
```

### External Execution for Match Offloading (`-e`)

With `-e`, you can use external commands to specify the range of holes. `-e` allows you to specify a shell pipeline as a string, which is executed in `/bin/sh`.

For instance, using a pipeline like `echo 3`, which outputs `3`, only the third line will be selected.

```bash
$ echo -e 'AAA\nBBB\nCCC' | teip -e 'echo 3'
AAA
BBB
[CCC]
```

It also works even if the output includes extraneous characters. For example, spaces or tab characters at the start of a line are ignored. Additionally, once a number is provided, non-numerical characters to the right of the number are disregarded.

```bash
$ echo -e 'AAA\nBBB\nCCC' | teip -e 'echo " 3"'
AAA
BBB
[CCC]
$ echo -e 'AAA\nBBB\nCCC' | teip -e 'echo " 3:testtest"'
AAA
BBB
[CCC]
```

Technically, the first captured group in the regular expression `^\s*([0-9]+)` is interpreted as a line number. `-e` will also recognize multiple numbers if provided across multiple lines.

```
$ echo -e 'AAA\nBBB\nCCC\nDDD\nEEE\nFFF' | teip -e 'seq 1 2 10' -- sed 's/. /@/g'
@@@
BBB
@@@
DDD
@@@
FFF
```

Note that the numbers must be in ascending order.

The pipeline receives the same standard input as `teip`. Here's a command using `grep` to print line numbers of a line containing "CCC" and the two following lines.

```
$ echo -e 'AAA\nBBB\nCCC\nDDD\nEEE\nFFF' | grep -n -A 2 CCC
3:CCC
4-DDD
5-EEE
```

Using this with `-e` punches holes in the line containing "CCC" and the two subsequent lines.

```
$ echo -e 'AAA\nBBB\nCCC\nDDD\nEEE\nFFF' | teip -e 'grep -n -A 2 CCC'
AAA
BBB
[CCC]
[DDD]
[EEE]
FFF
```

GNU `sed` has an `=` option, which prints the line number being processed. Below is an example to drill holes from the line containing "BBB" to "EEE".

```
$ echo -e 'AAA\nBBB\nCCC\nDDD\nEEE\nFFF' | teip -e 'sed -n "/BBB/,/EEE/="'
AAA
[BBB]
[CCC]
[DDD]
[EEE]
FFF
```

Similarly, you can perform these operations with `awk`.

```
$ echo -e 'AAA\nBBB\nCCC\nDDD\nEEE\nFFF' | teip -e 'awk "/BBB/,/EEE/{print NR}"'
```

Here's an example using `nl` and `tail` to make holes in the last three lines of input.

```
$ echo -e 'AAA\nBBB\nCCC\nDDD\nEEE\nFFF' | teip -e 'nl -ba | tail -n 3'
AAA
BBB
CCC
[DDD]
[EEE]
[FFF]
```

The `-e` argument is a single string, so pipes `|` and other symbols can be used as is.

EXAMPLES
-------

Replace 'WORLD' with 'EARTH' on lines containing 'HELLO'

```
$ cat file | teip -g HELLO -- sed 's/WORLD/EARTH/'
```

Edit the 2nd field of a CSV file

```
$ cat file.csv | teip --csv -f 2 -- tr a-z A-Z
```

Edit the 2nd, 3rd, and 4th fields of a TSV file

```
$ cat file.tsv | teip -D '\t' -f 2-4 -- tr a-z A-Z
```

Convert timestamps in /var/log/secure to UNIX time

```
$ cat /var/log/secure | teip -c 1-15 -- date -f- +%s
```

Edit lines containing 'hello' and the three lines before and after it

```
$ cat access.log | teip -e 'grep -n -C 3 hello' -- sed 's/./@/g'
```

SEE ALSO
--------

### Manual pages
cut(1), sed(1), awk(1), grep(1)

### Full documentation
<https://github.com/greymd/teip>

### Regular expression
https://docs.rs/regex/

### Regular expression (Oniguruma)
https://github.com/kkos/oniguruma/blob/master/doc/RE

### RFC 4180: Common Format and MIME Type for Comma-Separated Values (CSV) Files
https://www.rfc-editor.org/rfc/rfc4180

AUTHOR AND COPYRIGHT
------

Copyright (c) 2023 Yamada, Yasuhiro <yamada@gr3.ie> Released under the MIT License.
