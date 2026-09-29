#include <dobby.h>
#include <stddef.h>
#include <stdio.h>
#if defined(__ANDROID__)
#include <android/log.h>
#endif

__attribute__((noinline)) int fixture_target(int value) {
  __asm__ __volatile__("nop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\n"
                       "nop\nnop\nnop\nnop\nnop\nnop\nnop\nnop");
  return value + 7;
}

size_t fixture_layout(unsigned index) {
  switch (index) {
  case 0: return sizeof(DobbyRegisterContext);
  case 1: return _Alignof(DobbyRegisterContext);
  case 2: return offsetof(DobbyRegisterContext, general);
#if defined(__aarch64__)
  case 3: return offsetof(DobbyRegisterContext, general.regs.x0);
  case 4: return offsetof(DobbyRegisterContext, sp);
  case 5: return offsetof(DobbyRegisterContext, floating);
#elif defined(__arm__)
  case 3: return offsetof(DobbyRegisterContext, general.regs.r0);
  case 4: return offsetof(DobbyRegisterContext, sp);
#elif defined(__x86_64__) || defined(_M_X64)
  case 3: return offsetof(DobbyRegisterContext, general.regs.rax);
  case 4: return offsetof(DobbyRegisterContext, general.regs.rsp);
#elif defined(__i386__) || defined(_M_IX86)
  case 3: return offsetof(DobbyRegisterContext, general.regs.eax);
  case 4: return offsetof(DobbyRegisterContext, esp);
#endif
  default: return (size_t)-1;
  }
}

void fixture_log(const char *message) {
  puts(message);
  fflush(stdout);
#if defined(__ANDROID__)
  __android_log_write(ANDROID_LOG_INFO, "DobbyE2E", message);
#endif
}
