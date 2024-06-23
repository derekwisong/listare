#include <sys/ioctl.h>
#include <stdio.h>

int main(int argc, char** argv) {
  struct winsize w;

  if (ioctl(0, TIOCGWINSZ, &w) != 0) {
    perror("ioctl");
    return 1;
  }

  printf("TIOCGWINSZ: %x\n", TIOCGWINSZ);
  printf("lines %d\n", w.ws_row);
  printf("columns %d\n", w.ws_col);
  return 0;
}
