# Generating Columns

Would like to behave similarly to GNU `ls`.

## Determining Line Length

```c
/* Maximum number of columns ever possible for this display.  */
static size_t max_idx;

/* The minimum width of a column is 3: 1 character for the name and 2
   for the separating white space.  */
#define MIN_COLUMN_WIDTH	3

// later...
max_idx = MAX (1, line_length / MIN_COLUMN_WIDTH);

// later...
// line_length starts at 80
// then it checks the COLUMNS env var (which is the length of the term)
// if it is 
line_length = 80;
{
    char const *p = getenv ("COLUMNS");
    if (p && *p)
    {
    unsigned long int tmp_ulong;
    if (xstrtoul (p, NULL, 0, &tmp_ulong, NULL) == LONGINT_OK
        && 0 < tmp_ulong && tmp_ulong <= SIZE_MAX)
        {
        line_length = tmp_ulong;
        }
    else
        {
        error (0, 0,
            _("ignoring invalid width in environment variable COLUMNS: %s"),
                quotearg (p));
        }
    }
}

```

- (The MIN_COLUMN_WIDTH const)[https://github.com/wertarbyte/coreutils/blob/master/src/ls.c#L903]
- (The line_length variable)[https://github.com/wertarbyte/coreutils/blob/master/src/ls.c#L1586]

## Printing columns vertically

- See: https://github.com/wertarbyte/coreutils/blob/master/src/ls.c#L4284
