# Arcana

> It's not magic, it's talent and sweat.
> - Bertram Gilfoyle, _Silicon Valley_

Okay, fine, it's just sweat. **Arcana** is a templating engine intended
for static file generation. In theory, it could be used as a part of a larger
web templating framework, but it is by no means optimized for this usage.

1. [Filetype](#filetype)
2. [Getting and Setting](#getting-setting)
    1. [Set](#t-set)
    2. [Get](#t-get)
3. [Functions](#functions)
    1. [Fn](#t-fn)
4. [Whitespace Control](#whitespace-control)
5. [Comments](#comments)
6. [Arithmetic](#arithmetic)
    1. [Add](#t-add)
    2. [Sub](#t-sub)
    3. [Mul](#t-mul)
    4. [Div](#t-div)
    5. [Mod](#t-mod)
    6. [Pow](#t-pow)
7. [Property Accessors](#property-accessors)
    1. [Count](#t-count)
    2. [Length](#t-length)
    3. [Nth](#t-nth)
8. [Path Manipulation](#path-manipulation)
    1. [Path](#t-path)
    2. [Dirname](#t-dirname)
    3. [Basename](#t-basename)
9. [Nesting](#nesting)
    1. [Call](#t-call)
    2. [Compile](#t-compile)
    3. [Include](#t-include)
    4. [Extend](#t-extend)
10. [Control Flow](#control-flow)
    1. [Conditionals](#conditionals)
        1. [Assert](#t-assert)
        2. [If](#t-if)
    2. [For Loops](#for-loops)
        1. [For-Each](#t-foreach)
        2. [For-Split](#t-forsplit)
        3. [For-Dir](#t-fordir)
        4. [For-File](#t-forfile)
        5. [Loop Context](#loop-context)
11. [Glossary](#glossary)

## <a id="filetype"></a>Filetype

The expected filetype for Arcana is a UTF-8 encoded text file with the `arct`
extension. The file extension is optional, but it is the expected filetype of
the [official Vim syntax highlighting plugin](https://github.com/oh-clever/arcana.vim).

## <a id="getting-setting"></a>Getting and Setting

> I have a value that I want to use in multiple places without defining it multiple
> times.

```arct
{% set projectname %}Arcana{% /set %}\

# {{ projectname }}

**{{ projectname }}** is a templating engine intended for static file generation&hellip;
```

This would compile to the following [value](#g-value).

```md
# Arcana

**Arcana** is a templating engine intended for static file generation&hellip;
```

<a id="t-set"></a>The `set` tag compiles the encapsulated content
[content](#g-content) using an [unsealed](#g-unsealed) compiler and assigns the
resulting [value](#g-value) to the given [variable](#g-variable) in
[context](#g-context). Variable names (as well as function names) can contain upper
and lower case alphabetic characters [`a-zA-Z`], numbers (but cannot start with a
number) (`[0-9]`), the period character (`.`), and the underscore character (`_`). If
you're more of a regex person, then the valid pattern for a variable name is
`[a-zA-Z]([a-zA-Z0-9_.])*`.

The `set` tag also contains some more advanced behavior. It is list-y. Setting to
the same variable multiple times seemingly overwrites the value when using `get`,
but the value is also added to the end of a list of previous values. This assists
in constructing lists for loops.

<a id="t-get"></a>The `get` tag references a variable (or [function](#g-function))
and outputs the resulting value in-place.

## <a id="functions"></a>Functions

> I have content that I want to use in multiple places without defining it multiple
> times.

```arct
{% fn h2(anchor, text) %}\
	## <a id="{{ anchor }}"></a>{{ text }}\
{% /fn %}\

{{ h2("getting-setting", "Getting and Setting") }}

> I have a value that I want to use in multiple places&hellip;

{{ h2("functions", "Functions") }}

> I have content that I want to use in multiple places&hellip;
```

This would compile to the following value (you're going to get real sick of this phrase).

```md
## <a id="getting-setting"></a>Getting and Setting

> I have a value that I want to use in multiple places&hellip;

## <a id="functions"></a>Functions

> I have content that I want to use in multiple places&hellip;
```

<a id="t-fn"></a>The `fn` tag copies the encapsulated [content](#g-content) to
[context](#g-context) and when retrieved, assigns the passed arguments to the
pre-defined variables within the context of a [sealed](#sealed) compiler and places
the resulting value in-place.

## <a id="whitespace-control"></a>Whitespace Control

> What is the deal with these trailing backslashes in each `arct` example?

These are used for whitespace control. A backslash tells the compiler to ignore all
whitespace until the next non-whitespace character.

```arct
\     This \
\\    will \\\\\\\\\
\\\be \\\\\\
\\\               \\\\neater \
\\ 
than \
\\\\\     \\\\\    expected.
```

This would compille to the following value.

```txt
This will be neater than expected.
```

> Okay, that is all well and good; but how do I include a backslash character?

All Arcana syntax (including the whitespace control [backslash] character), must be
included using a separate file and the [`include`](#t-include) tag.

## <a id="comments"></a>Comments

> I want to say some stuff, but I don't want it to do some stuff.

Perfect use for a comment.

```arct
{# This is a comment.

It can be multiline.

It can contain another opening tag {#

But it will close on the first closing tag. #}\

Hello.
```

This would compile to the following value.

```txt
Hello.
```

## <a id="arithmetic"></a>Arithmetic

> I know this is all strings and stuff, but what if I need math?

Arcana supports addition, subtraction, multiplication, division, modulo, and
exponents. All variables are treated as strings. When arithmetic is needed, the
compiler will attempt to parse the string into a numeric value. The tags for each of
these operations functions in the same way. The value within the opening tag is the
left-most of the two numbers in the equation. The tag name, aka the operator, is the
operator itself that separates the two numbers. The content found within the tag is
the right-most of the two numbers.

<a id="t-add"></a>Addition is a good starting point.

```arct
{% add 4 %}2{% /add %}
```

This would compile to the following value.

```txt
6
```

> <a id="t-sub"></a>Okay, but what if I want to use a variable as the first number?

```arct
{% set x %}\
	{% add 4 %}\
		2\
	{% /add %}\
{% /set %}\

{% sub x %}3{% /sub %}
```

This would compile to the following value.

```txt
3
```

> <a id="t-mul"></a>Makes sense, but now I _also_ want to use a variable as the second number.

```arct
{% set x %}{% add 4 %}2{% /add %}{% /set %}\
{% set y %}{% sub x %}3{% /sub %}{% /set %}\

{% mul y %}{{ y }}{% /mul %}
```

This would compile to the following value.

```txt
9
```

> <a id="t-div"></a>Division?

```arct
{% div 8 %}4{% /div %}
```

This would compile to the following value.

```txt
2
```

> <a id="t-mod"></a>Modulo?

```arct
{% mod 5 %}2{% /mod %}
```

This would compile to the following value.

```txt
1
```

> <a id="t-pow"></a>Exponents?

```arct
{% pow 5 %}2{% /pow %}
```

This would compile to the following value.

```txt
25
```

## <a id="property-accessors"></a>Property Accessors

Being that [variables](#g-variable) are just listy amalgamations of strings, they
have some properties we might want to access.

<a id="t-length"></a>Strings have a number of characters.

```arct
{% set x %}This is a value.{% /set %}\

{% length x /%}
```

This would compile to the following value.

```txt
16
```

<a id="t-count"></a>Lists have a count of elements.

```arct
{% set x %}0{% /set %}\
{% set x %}1{% /set %}\
{% set x %}2{% /set %}\

{% count x /%}
```

This would compile to the following value.

```txt
3
```

In the case that a variable is unset, `count` would return 0.

<a id="t-nth"></a>Lists also have an element at each specific index.

```arct
{% set x %}Foo{% /set %}\
{% set x %}Bar{% /set %}\
{% set x %}Baz{% /set %}\

{% nth x %}1{% /nth %}
```

This would compile to the following value.

```txt
Bar
```

## <a id="path-manipulation"></a>Path Manipulation

Generating static files usually means that we're going to read static files as well.
In this case, it would be helpful to have some ability to manipulate paths.

<a id="t-path"></a>While working inside of the same directory, using a relative path
is perfectly fine, but what if we want to pass around the same [variable](#g-variable)
representing a relative path to multiple files, each of which may not live in the
same directory? This is the job for an absoltue path. The downside of this is that to
obtain an absolute path, the file-system-object **needs** to exist. Without it, the
compiler will return an error.

The following example assumes that the file `test.txt` exists in the same directory
as the example template (`/home/test/template/test.arct`).

```arct
{% path "./test.txt" /%}
```

This would compile to the following value.

```txt
/home/test/template/test.txt
```

Expanding upon this example, assume that `test.txt` exists in the directory
`content`, which is in the same directory as the same example template.

```arct
{% path "./test.txt" in "content" /%}
```

This would compile to the following value.

```txt
/home/test/template/content/test.txt
```


> <a id="t-dirname"></a>I have a path to a file, but I just need the directory.

```arct
{# /home/test/template/test.arct #}\
{% set file %}{% path "./test.txt" /%}{% /set %}\
{% dirname file /%}
```

This would compile to the following value.

```txt
/home/test/template
```

> <a id="t-basename"></a>I have a path to a file, but I just want the filename.

```arct
{# /home/test/template/test.arct #}\
{% set file %}{% path "./test.txt" /%}{% /set %}\
{% basename file /%}
```

This would compile to the following value.

```txt
test.txt
```

## <a id="nesting"></a>Nesting

> What is all of this path manipulation stuff about?

In a more complex static templating environment, templates in one directory may call
templates in another directory. The nested templates may rely on paths that were set
by from other directories and therefore must be absolute instead of relative. These
path manipulation tags become very useful when dealing with template nesting. Nesting
increases the reusability of your templates. Allowing them to be called from numerous
other templates. Keeping your environment
[DRY](https://en.wikipedia.org/wiki/Don%27t_repeat_yourself).

> This is getting a bit confusing in English. What is a nested template?

<a id="t-call"></a>The simplest form of nesting is to just include the compiled
[value](#g-value) and [context](#g-context) of one template file in another.

```arct
{# /home/test/template/fragments/header.arct #}\

<h1 class="section-header">Section: {{ name }}</h1>\
```

```arct
{# /home/test/template/page.arct #}\

{% set name %}Arcana Call Tag{% /set %}\
{% call "./fragments/header.arct" /%}
```

The file `page.arct` would compile to the following value.

```html
<h1 class="section-header">Section: Arcana Call Tag</h1>
```

The `call` tag uses an [unsealed](#g-unsealed) compiler. This leaves the entire
context exposed after it is complete. This makes it very useful for calling templates
that contain only [functions](#functions). Improving your separation of concerns and
template readability.

```arct
{# /home/test/template/function/header.arct #}\

{% fn header(name) %}\
    <h1 class="section-header">Section: {{ name }}</h1>\
{% /fn %}\
```

```arct
{# /home/test/template/page.arct #}\

{% call "function/header.arct" /%}\

{{ header("Arcana Nested Functions") }}
<p>Are very cool.</p>

{{ header("Tidy Code") }}
<p>Is also very cool.</p>
```

The file `page.arct` would compile to the following value.

```html
<h1 class="section-header">Section: Arcana Nested Functions</h1>
<p>Are very cool.</p>

<h1 class="section-header">Section: Tidy Code</h1>
<p>Is also very cool.</p>
```

> <a id="t-compile"></a>What if I only want the output value of the nested template
> and I want to throw out the context?

```arct
{# /home/test/template/fragments/header.arct #}\

{% set tagname %}h{{ taglevel }}{% /set %}\

<{{ tagname }} class="section-header">Section: {{ name }}</{{ tagname }}>\
```

```arct
{# /home/test/template/page.arct #}\

{% set taglevel %}1{% /set %}\
{% set name %}Arcana Compile Tag{% /set %}\
{% compile "./fragments/header.arct" /%}
{{ tagname }}
```

The file `page.arct` would compile to the following value.

```txt
<h1 class="section-header">Section: Arcana Compile Tag</h1>

```

Note the empty line representing the `tagname` [variable](#g-variable). This is
because the `compile` tag uses a [sealed](#g-sealed) compiler.

> <a id="t-include"></a>What about literal content? Can I have a file that is not
> compiled at all?

Assume the text file in this example is located at the path
`/home/test/template/resources/text.txt`.

```txt
Arcana whitespace control is performed using the "\" character.
The Arcana "set" tag looks like this: {% set test %}Hello, World!{% /set %}
```

```arct
{# /home/test/template/page.arct #}\

{% include "resources/text.txt" /%}
```

The file `page.arct` would compile to the following value.

```txt
Arcana whitespace control is performed using the "\" character.
The Arcana "set" tag looks like this: {% set test %}Hello, World!{% /set %}
```

One quirk of the `include` tag is the removal of the final newline character of the
file. This is because it would otherwise be impossible to include a file without it
ending in a new line on Linux systems.

> <a id="t-extend"></a>What if I have a template that acts a some sort of container?
> Like a predefined header, a dynamic body, and a predefined footer. How would I
> neatly nest this?

```arct
{# /home/test/template/base.arct #}\

---
page: {{ pageno }}
---

# {{ title }}

{{ CONTENT }}

`This document was templated using the Arcana templating engine.`
```

```arct
{# /home/test/template/page.arct #}\

{% extend "./base.arct" /%}\

{% set pageno %}1{% /set %}\
{% set title %}Test Page{% /set %}\

This is the content on the page. It will be placed right where the
`CONTENT` variable is retrieved.\
```

The file `page.arct` would compile to the following.

```md
---
page: 1
---

# Test Page

This is the content on the page. It will be placed right where the
`CONTENT` variable is retrieved.

`This document was templated using the Arcana templating engine.`
```

The `extend` tag defines a parent template to be compiled using an unsealed compiler
after the compilation of the current template is completed. The value output by the
child template is stored in context as the special variable `CONTENT`.

## <a id="control-flow"></a>Control Flow

> I wanna get fancy with conditional statements and loops for maximum templating
> madness.

Well, you're in luck. Arcana supports if-statements, assertions, and multiple types
of for-loops. Assertions and if-statements both support the same types of
<a id="conditionals"></a>logical conditions. These logical conditions are best
exemplified with assertions that will evaluate to true.

```arct
{# boolean-ish #}\
{% assert 1 /%}\
{% assert !0 /%}\

{# numeric #}\
{% assert 1 > 0 /%}\
{% assert 3 < 22 /%}\
{% assert 3 >= 2 /%}\
{% assert 4 <= 6 /%}\

{# strings #}\
{% assert "this" != "that" /%}\
{% assert "foo" == "foo" /%}\

{# chained and nested conditions #}\
{% assert !(34 <= 33) && (0 || 1) /%}\
{% assert ((0 || 1) && !(100 > 99 && !(6 > 5))) /%}\

true
```

This would compile to the following value.

```txt
true
```

There are also several unconventional boolean operators. These are `file`,
`directory`, and `exists`. Each of which checks the filesystem for the preceeding
value as a path.

```arct
{% assert "./this.txt" file /%}{# will return true if the path is a file #}\
{% assert "./this" directory /%}{# will return true if the path is a directory #}\
{% assert "./this" exists /%}{# will return true if the path is a file of directory #}\

true
```

This would compile to the following value given `./this.txt` is a file and `./this` is a directory.

```txt
true
```

<a id="t-assert"></a>While the preceeding examples for conditionals are also
sufficient for explaining the syntax of the `assert` tag, its usage still needs
defined. The `assert` tag enforces that a logical condition is true before
continuing. If this logical condition is false, the compiler displays an error and
exits. This can be useful in cases when you want the compiler to fail-fast or when
you want to ensure a template is not compiled by a build script (`{% assert 0 /%}`).

> <a id="t-if"></a>What if?

The `if`, `else if`, and `else` tags define branches of a template that will be
traversed by the compiler when the first true logical condition is encountered.

```arct
{% if 0 %}\
    Zero is true.\
{% else if 1 == 0 %}\
    Meaning no longer exists.\
{% else if 0 || 1 %}\
    All is right with the universe.\
{% else %}\
    The end is near.\
{% /if %}
```

This would compile to the following value.

```txt
All is right with the universe.
```

These potential branches are fully-parsed, but lazily-evaluated by an
[unsealed](#g-unsealed) compiler. This means that incorrect syntax in an unused
branch will still cause an error, but the incorrect usage of a variable or the calling
of an undefined function will not.

```arct
{% if 0 %}\
    {% set x %}0\
{% /if %}\

What?
```

The previous example would return an error.

```arct
{% if 0 %}\
    {% add x %}1{% /add %}\
{% /if %}\

Perfectly fine.
```

The previous example would compile to the following value, but changing the
if-condition to `true` would cause an error.

```txt
Perfectly fine.
```

> <a id="for-loops"></a>Let's say I have some things. Let's also say I want to do the
> same thing with all of my things. How do?

Well&hellip; what kinds of things? Even though Arcana's variables are stringy
nonsense, they are also listy. And it has a fixation on the filesystem. So there are
definitely some types of things.

> <a id="t-foreach"></a>How about those listy variables?

```arct
{% set x %}0{% /set %}\
{% set x %}1{% /set %}\
{% set x %}2{% /set %}\
{% set x %}3{% /set %}\

{% foreach i in x %}\
    {{ i }}\
{% /foreach %}
```

This would compile to the following value.

```txt
0123
```

> <a id="t-forsplit"></a>Let's say I have one variable and I want to split the string
> and iterate through the slices.

```arct
{% set list %}First, Second, Third, Fourth, Fifth{% /set %}\

{% forsplit word in list on ", " as loop reversed %}\
    {% if !loop.isfirst %}, {% /if %}{{ word }}\
{% /forsplit %}
```

This would compile to the following value.

```txt
Fifth, Fourth, Third, Second, First
```

This example introduces a couple new concepts. The `as loop` portion of the tag
assigns the [loop context](#loop-context) variables using the prefix `loop` (more on
loop context later). The `reversed` keyword does exact what you would think&mdash;it
reverses the items you are about to iterate over.

> <a id="t-forfile"></a>What about files?

Describing the usefulness of this is going to take an example that is a bit more
complex. Let's say we are inside of a directory with the following structure.

```txt
./
    function/
        file.arct
    page.arct
    resources/
        01.arct
        02.arct
        03.arct
        04.arct
```

And the file contents look like this.

```arct
{# ./function/file.arct #}\

{% fn file(path) %}\
    {% set content %}{% compile path /%}{% /set %}\
    {% set name %}{% basename path /%}{% /set %}\
    <h2>{{ name }}</h2>\
    <pre>{{ content }}</pre>\
{% /fn %}\
```

```arct
{# ./page.arct #}\

{% call "./function/file.arct" /%}\

{% forfile file in "./resources" from 1 to 3 as loop %}\
    {% set filepath %}{% path file /%}{% /set %}\

    {# Add a newline after each subsequent item #}\
    {% if !loop.isfirst %}
\   {% /if %}\

    {{ file(filepath) }}\
{% /forfile %}
```

```arct
{# ./resources/01.arct #}\

First\
```

```arct
{# ./resources/02.arct #}\

Second\
```

```arct
{# ./resources/03.arct #}\

Third\
```

```arct
{# ./resources/04.arct #}\

Fourth\
```

Compiling `./page.arct` would result in the following value.

```html
<h2>02.arct</h2><pre>Second</pre>
<h2>03.arct</h2><pre>Third</pre>
```

This example exposes another piece of loop functionality, the `from` and `to`
keywords. These define the item to on which to start and the item on which to
end, respectively. These keywords are also supported by each loop type.

> <a id="t-fordir"></a>And directories?

Let's say we are inside of a directory with the following structure.

```txt
./
    function/
        person.arct
    page.arct
    people/
        01/
            age.txt
            name.txt
        02/
            age.txt
            name.txt
```

And the file contents look like this.

```arct
{# function/person.arct #}\

{% fn get_person_name_path(person_dir) %}\
    {% path "name.txt" in person_dir /%}\
{% /fn %}\

{% fn get_person_name(person_dir) %}\
    {% set person_name_path %}{{ get_person_name_path(person_dir) }}{% /set %}\
    {% include person_name_path /%}\
{% /fn %}\

{% fn get_person_age_path(person_dir) %}\
    {% path "age.txt" in person_dir /%}\
{% /fn %}\

{% fn get_person_age(person_dir) %}\
    {% set person_age_path %}{{ get_person_age_path(person_dir) }}{% /set %}\
    {% include person_age_path /%}\
{% /fn %}\
```

```arct
{# page.arct #}\

{% call "./function/person.arct" /%}\

{% fordir person_dir in "./people" as loop %}\
    {% set person_dir_path %}{% path person_dir /%}{% /set %}\

    {% set name %}{{ get_person_name(person_dir_path) }}{% /set %}\
    {% set age %}{{ get_person_age(person_dir_path) }}{% /set %}\

    {# Assert age is a number #}\
    {% set ignore %}{% add age %}0{% /add %}{% /set %}\

    {% assert age >= 0 /%}\

    {% if !loop.isfirst %}
\   {% /if %}\

    <p>{{ name }} is {{ age }} year{% if age > 1 || age < 1 %}s{% /if %} old.</p>\
{% /fordir %}
```

`people/01/name.txt`

```txt
Fred
```

`people/01/age.txt`

```txt
33
```

`people/02/name.txt`

```txt
Mark
```

`people/02/age.txt`

```txt
35
```

Compiling `page.arct` would result in the following value.

```html
<p>Fred is 33 years old.</p>
<p>Mark is 35 years old.</p>
```

> <a id="loop-context"></a>You mentioned something about "loop context?"

When a loop defines a loop prefix using the `as` keyword, multiple variables are set
that help define the current state of the loop. Let's assume that the prefix defined
is `loop`.

1. `loop.index`: The current 0-based index of the iteration.
2. `loop.size`: The length of the items being iterated over.
3. `loop.isfirst`: Whether or not the loop is on the first iteration.
4. `loop.islast`: Whether or not the loop is on the last iteration.

# <a id="glossary"></a>Glossary

<a id="g-content">**Content**</a>: Valid Arcana syntax that can be compiled.

<a id="g-context">**Context**</a>: Functions and values currently accessible by name.

<a id="g-function">**Function**</a>: Content in context that will be compiled by a
sealed compiler when called.

<a id="g-sealed"></a>**Sealed**: A compiler whose context will be tossed-out with the
compiler once it is complete. The content within an `fn` tag is handled by a "sealed"
compiler.

```arct
{% fn this %}\
	{% set that %}That{% /set %}\
	This\
{% /fn %}\

{{ this() }}, {{ that }}.
```

```txt
This, .
```

<a id="g-unsealed"></a>**Unsealed**: A compiler whose context will be spilled when
the compiler is tossed-out once it is completed. The content within a `set` tag is
handled by an "unsealed" compiler.

```arct
{% set this %}\
	{% set that %}That{% /set %}\
	This\
{% /set %}\

{{ this }}, {{ that }}.
```

```txt
This, That.
```

<a id="g-value"></a>**Value**: Data that is either literal or output by a compiler.
The first addend passed to an `add` tag is a "value".

```arct
{% add 1 %}41{% /add %}
```

<a id="g-variable">**Variable**</a>: A value in context.
