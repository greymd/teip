<!--
Use md2man (https://github.com/sunaku/md2man) to generate the man file like this.
$ md2man-roff man.md > teip.1
-->
TEIP 1 "FEB 2023" "User Commands" ""
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
  Display this help and exit

`-V`, `--version`
  Show version and exit

`-g` <*pattern*>
  Bypassing lines that match the regular expression <*pattern*>

`-o`
  -g bypasses only matched parts

`-G`
  -g adopts Oniguruma regular expressions

`-f` <*list*>
  Bypassing these white-space separated fields

`-d` <*delimiter*>
  Use <*delimiter*> for field delimiter of -f

`-D` <*pattern*>
  Use a regular expression <*pattern*> for field delimiter of -f

`-c` <*list*>
  Select only these characters

`-e` <*string*>
  Execute <*string*> on another process that will receive identical standard input as the teip, and numbers given by the result are used as line numbers for bypassing

`-l` <*list*>
  Bypassing these lines

`--csv`
  -f interprets <list> as field number of a CSV according to RFC 4180, instead of white-space separated fields

`-s`
  Execute new command for each bypassed chunk

`--chomp`
  Command spawned by -s receives standard input without trailing newlines

`-v`
  Invert the sense of selecting

`-z`
  NUL is used as line delimiter instead of the newline

-A <*number*>
  Use  together with `-g <pattern>`.
  Alias of `-e 'grep -n -A <number> <pattern>'`

-B <*number*>
  Use  together with `-g <pattern>`.
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

*command* is the command and its arguments that `teip` executes.
*command* must print a single line of result for each line of the input.
In the simplest example, the cat(1) command always succeeds.
Because the cat prints the same number of lines against the input.

```
$ echo ABCDEF | teip -og . -- cat
ABCDEF
```

sed(1) works with the typical pattern.

```
$ echo ABCDEF | teip -og . -- sed 's/[ADF]/@/'
@BC@E@
```

If the rule is not satisfied, the result will be inconsistent.
For example, the grep(1) may fail. Here is an example.

```
$ echo ABCDEF | teip -og . -- grep '[ABC]'
ABC
teip: Output of given command is exhausted
```

`teip` could not get the result corresponding to D, E, and F. That is why the example fails.
If such the inconsistency occurs, `teip` will exit with the error message. Then, the exit status will be 1.

```
$ echo $?
1
```

If *command* is not given, `teip` shows how standard input will be devided into chunks.

```
$ echo ABCDEF | teip -og .
[A][B][C][D][E][F]
```

### *list*

*list* is an expression to specify the range of fields or characters.
The notation is compatible with the one used in cut(1). Refer to the cut(1) manual in detail.
Here are some examples.

Select 1st, 3rd, and 5th fields.

```
$ echo 1 2 3 4 5 | teip -f 1,3,5 -- sed 's/./@/'
@ 2 @ 4 @
```

Select fields between 2nd and 4th.

```
$ echo 1 2 3 4 5 | teip -f 2-4 -- sed 's/./@/'
1 @ @ @ 5
```

Select all the fields after 3rd.

```
$ echo 1 2 3 4 5 | teip -f 3- -- sed 's/./@/'
1 2 @ @ @
```

Select all the fields before 4th.

```
$ echo 1 2 3 4 5 | teip -f -4 -- sed 's/./@/'
@ @ @ @ 5
```

### *pattern*

*pattern* is a regular expression whose grammar follows "regex crate".
Refer to the link in *SEE ALSO* about the details.

### Necessity of **--**

`teip` interprets arguments after `--` as *command* and its argument.

If **--** is omitted, the command fails in this example.

```
$ echo "100 200 300 400" | teip -f 3 cut -c 1
teip: Invalid arguments.
```

This is because the `cut` uses the `-c` option. The option of the same name is also provided by `teip`, which is confusing.

```
$ echo "100 200 300 400" | teip -f 3 -- cut -c 1
100 200 3 400
```

### External execution for match offloading (`-e`)

With `-e`, you can use the external commands you are familiar with to specify the range of holes.
`-e` allows you to specify the shell pipeline as a string. This pipeline is executed in `/bin/sh`.

For example, with a pipeline `echo 3` that outputs `3`, then only the third line will be bypassed.

```bash
$ echo -e 'AAA\nBBB\nCCC' | teip -e 'echo 3'
AAA
BBB
[CCC]
```

It works even if the output is somewhat 'dirty'.
For example, if any spaces or tab characters are included at the beginning of a line, they are ignored.
Also, once a number is given, it does not matter if there are non-numerical characters to the right of the number.

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

Technically, the first captured group in the regular expression `^\s*([0-9]+)` is interpreted as a line number.
`-e` will also recognize multiple numbers if the pipeline provides multiple lines of numbers.

```
$ echo -e 'AAA\nBBB\nCCC\nDDD\nEEE\nFFF' | teip -e 'seq 1 2 10' -- sed 's/. /@/g'
@@@
BBB
@@@
DDD
@@@
FFF
```

Note that the order of the numbers must be in ascending order.

The pipeline obtains identical standard input as `teip`.
The following command is a `grep` command that prints **the line numbers of the line containing the string "CCC" and the two lines after it**.

```
$ echo -e 'AAA\nBBB\nCCC\nDDD\nEEE\nFFF' | grep -n -A 2 CCC
3:CCC
4-DDD
5-EEE
```

If you give this command to `-e`, you can punch holes in **the line containing the string "CCC" and the two lines after it**.

```
$ echo -e 'AAA\nBBB\nCCC\nDDD\nEEE\nFFF' | teip -e 'grep -n -A 2 CCC'
AAA
BBB
[CCC]
[DDD]
[EEE]
FFF
```

GNU `sed` has `=`, which prints the line number being processed.
Below is an example of how to drill from the line containing "BBB" to the line containing "EEE".

```
$ echo -e 'AAA\nBBB\nCCC\nDDD\nEEE\nFFF' | teip -e 'sed -n "/BBB/,/EEE/="'
AAA
[BBB]
[CCC]
[DDD]
[EEE]
FFF
```

Of course, similar operations can also be done with `awk`.

```
$ echo -e 'AAA\nBBB\nCCC\nDDD\nEEE\nFFF' | teip -e 'awk "/BBB/,/EEE/{print NR}"'
```

The following is an example of combining the commands `nl` and `tail`.
You can only make holes in the last three lines of input.

```
$ echo -e 'AAA\nBBB\nCCC\nDDD\nEEE\nFFF' | teip -e 'nl -ba | tail -n 3'
AAA
BBB
CCC
[DDD]
[EEE]
[FFF]
```

The `-e` argument is a single string.
Therefore, pipe `|` and other symbols can be used as it is.


EXAMPLES
-------


Replace 'WORLD' to 'EARTH' on lines containing 'HELLO'

```
$ cat file | teip -g HELLO -- sed 's/WORLD/EARTH/'
```

Edit 2nd field of the CSV file

```
$ cat file.csv | teip --csv -f 2 -- tr a-z A-Z
```

Edit 2nd, 3rd and 4th fields of TSV file

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
