# Arcana

> It's not magic, it's talent and sweat.
> - Bertram Gilfoyle, _Silicon Valley_

Okay, fine, maybe it's just sweat. **Arcana** is a templating engine intended
for static file generation. In theory, it could be used as a part of a larger
web templating framework, but it is by no means optimized for this usage.

## <a id="tags"></a>Tags

### <a id="t-add"></a>Add

Sums an addend stored in [context](#g-context) or a literal addend and a
templated addend. The following example uses an addend stored in context.

```arcana
{% set x %}5{% /set %}\
{% add x %}10{% /add %}
```

```txt
15
```

The following other tag(s) were used in this example.

- [_set_](#t-set)

The following example uses a literal addend, `12` in this case. The number could
also appear inside of double quotes.

```arcana
{% add 12 %}4{% /add %}
```

```txt
16
```

### <a id="t-assert"></a>Assert

Verifies that a [condition](#conditions) is truthy before continuing, will throw
at compile-time otherwise.

```arcana
{% assert "1" /%}
```

### <a id="t-basename"></a>Basename

Canonicalizes a literal path or a path from [context](#g-context) and retrieves
the basename. The path **must** exist or an error will be thrown at compile time.
The following example uses a literal path.

```arcana
{% basename "./this/file.txt" /%}
```

```txt
file.txt
```

The following example uses a path from [context](#g-context).

```arcana
{% set file %}./this/file.txt{% /set %}\
{% basename file /%}
```

```txt
file.txt
```

The following other tag(s) were used in this example.

- [_set_](#t-set)

### <a id="t-call"></a>Call

Processes an external file inline, modifying the existing [context](#g-context)
along the way.

```arcana
{# ./functions/header.arct #}\
{% fn header(lvl, txt) %}\
    <h{{ lvl }}>{{ txt }}</h{{ lvl }}>\
{% /fn %}\
```

```arcana
{% call "./functions/header.arct" /%}\
{{ header("2", "Hello") }}
```

```html
<h2>Hello</h2>
```

The following other tag(s) were used in this example.

- [_get_](#t-get)
- [_fn_](#t-fn)

### <a id="comment"></a>Comment

Instructs the compiler to skip all content contained within the open/close tags.

```arcana
{# this is a comment #}
```

### <a id="t-compile"></a>Compile

Processes an external file inline without modifying the existing
[context](#g-context).

```arcana
{# ./set/name.arct #}\
{% set name %}Fred{% /set %}{{ name }}, \
```

```arcana
{% set name %}Mark{% /set %}\
{% compile "./set/name.arct" /%}{{ name }}
```

```txt
Fred, Mark
```

The following other tag(s) were used in this example.

- [_set_](#t-set)

### <a id="t-count"></a>Count

Counts the number of values set to a variable from context. If the variable
does not exist, the value returned will be zero.

```arcana
{% set x %}One{% /set %}\
{% set x %}Two{% /set %}\
{% count x /%}
```

```txt
2
```

The following other tag(s) were used in this example.

- [_set_](#t-set)

### <a id="t-dirname"></a>Dirname

Canonicalizes a literal path or a path from [context](#g-context) and retrieves
the dirname. The path **must** exist or an error will be thrown at compile time.
The following example uses a literal path.

```arcana
{# assume this file exists at "/home/user/file.txt" #}\
{% dirname "./this/file.txt" /%}
```

```txt
/home/user/this
```

The following example uses a path from [context](#g-context).

```arcana
{# assume this file exists at "/home/user/file.txt" #}\
{% set file %}./this/file.txt{% /set %}\
{% dirname file /%}
```

```txt
/home/user/this
```

The following other tag(s) were used in this example.

- [_set_](#t-set)

### <a id="t-div"></a>Div

Performs division on a dividend in [context](#g-context) or a literal dividend
and a templated divisor. The following example uses a dividend in context.

```arcana
{% set x %}4{% /set %}\
{% div x %}2{% /div %}
```

```txt
2
```

The following other tag(s) were used in this example.

- [_set_](#t-set)

The following example uses a literal dividend.

```arcana
{% div 12 %}4{% /set %}
```

```txt
3
```

### <a id="t-extend"></a>Extend

Sets a single file as an outer template to process with the result of the
current file. The [context](#g-context) will be passed along and the
[content](#g-content) will be assigned to the special [context](#g-context)
[variable](#g-variable) `CONTENT`. If the extend tag is used multiple times
within the same template, the last tag used wins.

```arcana
{# ../papa.arct #}\

{% assert name /%}\
{% assert paragraph /%}\

<h1>{{ name }}</h1>
<p>{{ paragraph }}</p>\

{% if CONTENT %}
<hr>
<p>{{ CONTENT }}</p>\
{% /if %}
```

```arcana
{% extend "../papa.arct" /%}\

{% set name %}Fred{% /set %}\
{% set paragraph %}This is a paragraph.{% /set %}\

And here is some output content.
```

```html
<h1>Fred</h1>
<p>This is a paragraph.</p>
<hr>
<p>And here is some output content.</p>
```

The following other tag(s) were used in this example.

- [_assert_](#t-assert)
- [_if_](#t-if)
- [_set_](#t-set)

### <a id="t-fn"></a>Fn

Registers a [function](#g-function) in [context](#g-context) which can be called
using the [get](#t-get) tag. A function can have anywhere from 0 to _n_
arguments.

```arcana
{% fn commas(one, two, three) %}\
    {{ one }}, {{ two }}{% if three %}, {{ three }}{% /if %}.\
{% /fn %}\
{{ commas("First", "Second", "Third") }}
{{ commas("First", "Second") }}
```

```txt
First, Second, Third.
First, Second.
```

The following other tag(s) were used in this example.

- [_get_](#t-get)
- [_if_](#t-if)

### <a id="t-loops"></a>Loops

Below is a generic syntax applicable to each type of loop.

```txt
{% forTYPE ITEM COLLECTION [from START] [to END] [as CTX] [reversed] %}
    CONTENT
{% else %}
    No items.
{% /forTYPE %}
```

Loops will iterate through a `COLLECTION` and generate the encapsulated `CONTENT`
for each item in the collection. The `TYPE` of collection iterated over is
dependent upon the specific loop tag used. Each type of loop can specify a
variable to contain [`CTX`](#loop-context) so that templating can be
performed based on details regarding the loop's state. Each type of loop can
specify a `START` and `END` index by using the `from` and `to` keywords
respectively.  The values of `from` and `to` can be literals or from
[context](#g-context). Each type of loop can also specify the `reversed`
keyword to iterate through the collection backwards. Each loop can specify
an optional `else` block which will trigger when the collection is empty.

#### <a id="t-fordir"></a>Fordir

Loops through each directory within a given directory. The element
[variable](#g-variable) will contain the path of the directory.

Assume the following file stucture for the next example.

```txt
./
 \
  a-dir/
       \
        First/
        Second/
        Third/
```

```arcana
{% fordir d in "./a-dir" as dir_loop %}\
    {% if dir_loop.isfirst %}{% else %}, {% /if %}\
    "{{ d }}"\
{% else %}\
    {# no directories in "./a-dir" #}\
{% /fordir %}
```

```txt
"./a-dir/First", "./a-dir/Second", "./a-dir/Third"
```

The following other tag(s) were used in this example.

- [_if_](#t-if)

#### <a id="t-foreach"></a>Foreach

Loops through each value in a given variable in [context](#g-context). See
[set](#t-set) for info on how a [variable](#g-variable) can have multiple
values.

```arcana
The siblings are \
{% set names %}Mark{% /set %}\
{% set names %}Fred{% /set %}\
{% set names %}Karissa{% /set %}\
{% foreach name in items as name_loop %}\
    {% if name_loop.isfirst %}{% else %}, {% /if %}\
        {% if name_loop.islast %}and {% /if %}\
        {{ name }}\
    {% /if %}\
    {% if name_loop.islast %}.{% /if %}\
{% else %}\
    {# no items #}
{% /foreach %}
```

```txt
The siblings are Mark, Fred, and Karissa.
```

The following other tag(s) were used in this example.

- [_if_](#t-if)
- [_set_](#t-set)

#### <a id="t-forfile"></a>Forfile

Loops through each file in a given directory. The element
[variable](#g-variable) will contain the path of the file.

Assume the following file stucture and contents for the next example.

```txt
./
 \
  sibling.arct
  siblings/
          \
           first.arct
           second.arct
           third.arct
```

```arcana
{# ./sibling.arct #}\
{% assert sibling.filepath /%}\
{% call sibling.filepath /%}\
{% assert sibling.name /%}\
{% assert sibling.description /%}\
<tr><td>{{ sibling.name }}</td><td>{{ sibling.description }}.</td></tr>\
```

```arcana
{# ./siblings/first.arct #}\
{% set sibling.name %}Mark{% /set %}\
{% set sibling.description %}The elder{% /set %}\
```

```arcana
{# ./siblings/second.arct #}\
{% set sibling.name %}Fred{% /set %}\
{% set sibling.description %}The poor middle-child{% /set %}
```

```arcana
{# ./siblings/third.arct #}\
{% set sibling.name %}Karissa{% /set %}\
{% set sibling.description %}Da baby{% /set %}\
```

```arcana
<table>
    <thead>
        <tr>
            <th>Name</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>\
        {% set sibsdir %}{% path "./siblings" /%}{% /set %}\
        {% set sibtemplate %}{% path "./sibling.arct" /%}{% /set %}\
        {% forfile sibling.filepath in sibsdir %}
        {% compile sibtemplate /%}\
        {% else %}\
            {# no files in "./a-dir" #}\
        {% /fordir %}
    </tbody>
</table>
```

```html
<table>
    <thead>
        <tr>
            <th>Name</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr><td>Mark</td><td>The elder.</td></tr>
        <tr><td>Fred</td><td>The poor middle-child.</td></tr>
        <tr><td>Karissa</td><td>Da baby.</td></tr>
    </tbody>
</table>
```

The following other tag(s) were used in this example.

- [_call_](#t-call)
- [_compile_](#t-compile)
- [_path_](#t-path)
- [_set_](#t-set)

#### <a id="t-forsplit"></a>Forsplit

Loop through sections of a string split on a given delimiter. The
[variable](#g-variable) will contain the current section. The string value can
be provided literally or from [context](#g-context). The delimiter can be
provided literally or from [context](#g-context) as well.

```arcana
{% forsplit number in "0,1,2,3,4" on "," from "1" to "4" as loop reversed %}\
    {% if !loop.isfirst %}, {% /if %}{{ number }}\
{% /forsplit %}
```

```txt
4, 3, 2, 1
```

### <a id="t-get"></a>Get

Gets a value from a [variable](#g-variable) in [context](#g-context) or calls
a function in [context](#g-context). The following example gets a value from
[context](#g-context).

```arcana
{% set msg %}Hi{% /set %}\
{{ msg }}
```

```txt
Hi
```

The following other tag(s) were used in this example.

- [_set_](#t-set)

The following example calls a function in [context](#g-context).

```arcana
{% fn commas(a, b, c) %}\
    {{ a }}, {{ b }}{% if c %}, {{ c }}{% /if %}\
{% /fn %}\

{% set d %}foo{% /set %}\
{% set e %}bar{% /set %}\

{{ commas(d, e, "baz") }}
{{{ commas(d, e) }}
```

```txt
foo, bar, baz
foo, bar
```

The following other tag(s) were used in this example.

- [_fn_](#t-fn)
- [_set_](#t-set)

### <a id="t-if"></a>If

Compiles one of two code-paths depending on whether the [condition](#conditions)
evaluates to true or false. The `else` tag is an optional inclusion.

```arcana
{% if "1" %}\
    True\
{% else %}\
    False\
{% /if %}
```

```txt
True
```

### <a id="t-include"></a>Include

Includes a file inline with no compilation. Useful for including files which
contain `Ten Plates` syntax.

```arcana
{# ./includes/file.arct #}\
{% set name %}Fred{% /set %}\
```

```arcana
{% include "./includes/file.arct" /%}
```

```txt
{# ./includes/file.arct#}\
{% set name %}Fred{% /set %}\
```

The following other tag(s) were used in this example.

- [_set_](#t-set)

### <a id="t-length"></a>Length

Counts the number of characters in a literal value or a value from context. If
the variable does not exist, the value returned will be zero. The following
example uses a value from context.

```arcana
{% set x %}Something{% /set %}\
{% length x /%}
```

```txt
9
```

The following other tag(s) were used in this example.

- [_set_](#t-set)

The following example uses a literal value.

```arcana
{% length "Something" /%}
```

```txt
9
```

### <a id="t-mod"></a>Mod

Performs modulo operation on a dividend in [context](#g-context) or a literal
dividend and a templated divisor. The following example uses a dividend from
context.

```arcana
{% set x %}4{% /set %}\
{% mod x %}2{% /mod %}
```

```txt
0
```

The following other tag(s) were used in this example.

- [_set_](#t-set)

The following example uses a literal dividend.

```arcana
{% mod 4 %}2{% /mod %}
```

```txt
0
```

### <a id="t-mul"></a>Mul

Performs multiplication on a multiplicand in [context](#g-context) or a literal
multiplicand and a templated multiplier. The following example uses a
multiplicand from context.

```arcana
{% set x %}4{% /set %}\
{% mul x %}2{% /mul %}
```

```txt
8
```

The following other tag(s) were used in this example.

- [_set_](#t-set)

The following example uses a literal multiplicand.

```arcana
{% mul 8 %}2{% /mul %}
```

```txt
16
```

### <a id="t-nth"></a>Nth

Retrieves the _n_-th element from an array of values.

```arcana
{% set arr %}One{% /set %}\
{% set arr %}Two{% /set %}\
{% set arr %}Three{% /set %}\
{% nth arr %}1{% /nth %}
```

```txt
Two
```

The following other tag(s) were used in this example.

- [_set_](#t-set)

### <a id="t-path"></a>Path

Computes the canonical path for a given path. The entry **must** exist in the
file system to avoid throwing an error.

```arcana
{# imagine this file exists at "/home/user/file.arct" #}\
{% path "./file.txt" /%}
```

```txt
/home/user/file.txt
```

### <a id="t-set"></a>Set

Sets a value for a [variable](#g-variable) in [context](#g-context). When
multiple values are set for a given [variable](#g-variable), the previous value
is not overwritten, but is masked by the new value. These values can then be
iterated over in the order in which they were set using the
[foreach](#t-foreach) tag.

```arcana
{% set v %}1{% /set %}\
{{ v }}
```

```txt
1
```

The following other tag(s) were used in this example.

- [_get_](#t-get)

### <a id="t-sub"></a>Sub

Performs subtraction on a minuend in [context](#g-context) or a literal minuend
and a templated subtracahend. The following example uses a minuend in context.

```arcana
{% set x %}4{% /set %}\
{% sub x %}2{% /sub %}
```

```txt
2
```

The following other tag(s) were used in this example.

- [_set_](#t-set)

The following example uses a literal minuend.

```arcana
{% sub 5 %}4{% /sub %}
```

```txt
1
```

## <a id="conditions"></a>Conditions

A set of one or more of logical assertions evaluating to true or false. These
can be nested using parenthetical notation or conjoined using the
short-circuiting _and_ or _or_ operators and negated using the _not_ operator.
The values contained within conditions are evaluated in their _string_ form so
`Ten Plates` performs boolean casting on all values.

```arcana
{# true #}{% assert "1" /%}
{# true #}{% assert "Hello, World!" /%}

{# false #}{% assert "0" /%}
{# false #}{% assert "" /%}
{# false #}{% assert a /%}

{# true #}{% set a %}1{% /set %}
{# true #}{% assert "1" == a /%}

{# true #}{% set b %}0{% /set %}
{# true #}{% assert a || b /%}

{# true #}{% assert (a && b) || "1" /%}

{# true #}{% set d %}500{% /set %}
{# true #}{% assert d > a /%}

{# true #}{% assert "501" > d /%}

{# true #}{% assert "501" >= d /%}

{# true #}{% assert "501" != d /%}

{# true #}{% assert "501" <= d /%}
{# true #}{% assert "501" < d /%}

{# false #}{% assert !("501" <= d) /%}
```

## <a id="loop-context"></a>Loop Context

The optional loop context contains useful information regarding the state of
the loop.

`index`: The current index of the iteration. Zero indexed.

`size`: The length of the collection being iterated over.

`isfirst`: Whether or not the current iteration is the first.

`islast`: Whether or not the current iteration is the last.

## <a id="glossary"></a>Glossary

<a id="g-content">**Content**</a>: The final output of a template.

<a id="g-context">**Context**</a>: Functions, values, and other data currently
in-scope and usable.

<a id="g-function">**Function**</a>: A block of Arcana keyed with a given
name for future retrieval and compilation against an optional set of named
arguments.

<a id="g-variable">**Variable**</a>: A value in context keyed with a given
name for future retrieval.
