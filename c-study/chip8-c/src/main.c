#include <raylib.h>

#define MEMORY_SIZE 4096

int mem[(MEMORY_SIZE)];

int main(void) {
  InitWindow(768, 384, "chip8-c");
  SetTargetFPS(60);

  while (!WindowShouldClose()) {
    BeginDrawing();
    ClearBackground(BLACK);

    EndDrawing();
  }

  CloseWindow();
  return 0;
}
