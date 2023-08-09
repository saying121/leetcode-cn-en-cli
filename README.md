# A toy project

- 【[中文文档](./README-CN.md)】


<!--toc:start-->
- [A toy project](#a-toy-project)
  - [Install](#install)
  - [Useage](#useage)
  - [Configuration](#configuration)
    - [First](#first)
    - [Here are the explanations for each field](#here-are-the-explanations-for-each-field)
<!--toc:end-->

## Install

```shell
cargo install lcode
```

## Useage

Generate configuration, manual modification of the configuration is also possible,
and it will be automatically generated at runtime.
Without -c, it will be generated in English.

```shell
lcode gencon -c
```

Synchronize basic data first.

```shell
lcode sync
```

View the documentation for assistance.

```shell
lcode -h
```

Begin selecting a question.

```shell
lcode fzy <edit>
```

https://github.com/saying121/leetcode-cn-en-cli/assets/74663483/57a633e5-6bae-4816-a224-d7d61d2141af

## Configuration

The configuration is located at `~/.config/leetcode-cn-en-cli/config.toml`

```toml
tongue = "en"
column = 4
num_sublist = 10
page_size = 25
editor = ["vim"]
lang = "rust"
code_dir = "/home/user/.local/share/leetcode-cn-en-cli"

[urls]
origin = "https://leetcode.cn"
graphql = "https://leetcode.cn/graphql"
all_problem_api = "https://leetcode.cn/api/problems/$category"
submit = "https://leetcode.cn/problems/$slug/submit/"
test = "https://leetcode.cn/problems/$slug/interpret_solution/"
submissions = "https://leetcode.cn/submissions/detail/$id/check/"
favorites = "https://leetcode.cn/list/api/questions"

[support_lang]
langs = ["rust", "bash", "c", "cpp", "csharp", "golang", "java", "javascript", "kotlin", "mysql", "php", "python", "python3", "ruby", "scala", "swift", "typescript", "racket", "erlang", "elixir", "dart"]

[cookies]
csrf = ""
session = ""
```

### First

Press <kbd>F12</kbd> on the browser's `leetcode.com/cn` page,
Find the cookie field, copy the **csrf** and **session** sections inside it into the configuration.

### Here are the explanations for each field

Fill in either **cn** or **en**, with **en** being the default.

```toml
tongue = "en"
```

---

When retrieving the **submissionlist**, how many columns should be displayed.

```toml
column = 4
```

---

How many recent entries of the submissionlist information should be displayed.

```toml
num_sublist = 10
```

---

How many questions should be displayed at once when interactively selecting a question.

```toml
page_size = 25
```

---

Fill in your editor, it will attempt to retrieve it from the environment variables EDITOR and VISUAL,
otherwise it will default to vim.

```toml
editor = ["vim"]
```

You can add additional parameters at the end.

```toml
editor = ["vim", "--noplugin"]
```

---

Set your selected programming language.

```toml
lang = "rust"
```

---

Set the location for storing code and test cases.

```toml
code_dir = "/home/user/.local/share/leetcode-cn-en-cli"
```
