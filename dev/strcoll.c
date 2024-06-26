#include <stdio.h>
#include <string.h>
#include <locale.h>

int main()
{
    const char* s1 = "Android";
    const char* s2 = ".android";
    const char* s3 = "android-studio";
    
    const char* setloc = setlocale(LC_ALL, "");

    printf("setlocale = %s\n", setloc);

    printf("strcmp(Android, .android) = %d\n", strcmp(s1, s2));
    printf("strcmp(Android, android-studio) = %d\n", strcmp(s1, s3));
    printf("strcmp(.android, android-studio) = %d\n", strcmp(s2, s3));

}