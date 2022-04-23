<!--
Use md2man (https://github.com/sunaku/md2man) to generate the man file like this.
$ md2man-roff man.md > teip.1
-->
TEIP 1 "MAY 2020" "User Commands" ""
=======================================

NAME
----

teip - Highly efficient "Masking tape" for standard input

SYNOPSIS
--------

`teip` -g <*pattern*> [-oGsvz] [--] [<*command*>...]

`teip` -f <*list*> [-d <*delimiter*> | -D <*pattern*>] [-svz] [--] [<*command*>...]

`teip` -c <*list*> [-svz] [--] [<*command*>...]

`teip` --help | --version

DESCRIPTION
-----------
Only a selected part of standard input is passed to any command for execution.

OPTIONS
-------
`--help`
  Display this help and exit

`--version`
  Show version and exit

`-g` <*pattern*>
  Select lines that match the regular expression <*pattern*>

`-o`
  -g selects only matched parts

`-G`
  -g adopts Oniguruma regular expressions

`-f` <*list*>
  Select only these white-space separated fields

`-d` <*delimiter*>
  Use <*delimiter*> for field delimiter of -f

`-D` <*pattern*>
  Use a regular expression <*pattern*> for field delimiter of -f

`-c` <*list*>
  Select only these characters

`-e` <*string*>
  Execute <*string*> on another process that receives exactly the same standard input as the teip, and numbers given by the the result is used as line numbers for bypassing

`-s`
  Execute command for each selected part

`-v`
  Invert the sense of selecting

`-z`
  NUL is used as line delimiter instead of the newline

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

If *command* is not given, `teip` prints how standard input is tokenized.

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

EXAMPLES
-------

Edit 2nd, 3rd, and 4th columns in the CSV file

```
$ cat file.csv | teip -f 2-4 -d , -- sed 's/./@/g'
```

Convert timestamps in /var/log/secure to UNIX time

```
$ cat /var/log/secure | teip -c 1-15 -- date -f- +%s
```

Edit the line containing 'hello' and the three lines before and after it

```
$ cat access.log | teip -e 'grep -n -C 3 hello' -- sed 's/./@/g'
```

SEE ALSO
--------

### Manual pages
cut(1)

### Full documentation
<https://github.com/greymd/teip>

### Regular expression
https://docs.rs/regex/

### Regular expression (Oniguruma)
https://github.com/kkos/oniguruma/blob/master/doc/RE

AUTHOR AND COPYRIGHT
------

Copyright (c) 2022 Yamada, Yasuhiro <yamada@gr3.ie> Released under the MIT License.
