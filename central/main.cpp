#include <wdm.h>

extern "C" {

int _fltused = 0;

void *__CxxFrameHandler3([[maybe_unused]] void *pExcept,
                         [[maybe_unused]] void *pRN,
                         [[maybe_unused]] void *pContext,
                         [[maybe_unused]] void *pDC) {
  ASSERT(false);
  return nullptr;
}

}
