#if defined(WIN32) || defined(WIN64)
#warning This is Windows
#else
#warning This is Linux
#endif

#if defined(i386) || defined(__i386) || defined(__i386__) || defined(_M_IX86) || defined(_X86_) || defined(__X86__)
#warning This is x86
#elif defined(__amd64__) || defined(__amd64) || defined(__x86_64__) || defined(__x86_64) || defined(_M_AMD64) || defined(_M_X64) || defined(_WIN64) || defined(WIN64)
#warning This is amd64
#elif defined(__arm__) || defined(_M_ARM)
#warning This is ARM
#if defined(__LP64__) || defined(_LP64)
#warning This is LP
#else
#warning I don't know what architecture this is
#endif
#endif