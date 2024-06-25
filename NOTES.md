# Generating Columns

Would like to behave similarly to GNU `ls`.

(ls.c source code)[https://github.com/coreutils/coreutils/blob/master/src/ls.c]

## Various requirements

- Failure to stat a command line argument leads to an exit status of 2. For othher files,
  stat failure provokes an exit status of 1.
  (source)[https://github.com/wertarbyte/coreutils/blob/master/src/ls.c#L2824C1-L2826C47]
