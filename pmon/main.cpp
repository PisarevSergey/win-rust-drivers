#include <wdm.h>

extern "C" NTSTATUS DriverEntry(PDRIVER_OBJECT, PUNICODE_STRING)
{
    return STATUS_UNSUCCESSFUL;
}
