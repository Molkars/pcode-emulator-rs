#include <stdint.h>
#include <vector>
#include <string>
#include <memory>

#include "sleigh/loadimage.hh"
#include "sleigh/sleigh.hh"
#include "sleigh/translate.hh"
#include "rust/cxx.h"

using std::unique_ptr;

// The load image is responsible for retrieving instruction bytes, based on address, from a binary executable.
class RustLoadImage : public LoadImage {
public:
    uintb base_address;
    int4 length;
    const uint8_t *bytes;

    RustLoadImage();

    void setBytes(uintb base_address, const uint8_t *bytes, int4 len);
    void loadFill(uint1 *ptr, int4 size, const Address &addr);

    virtual string getArchType(void) const;
    virtual void adjustVma(long);
};

class DisassemblyInstruction {
public:
    Address addr;
    uint64_t len;
    std::string mnem;
    std::string body;

    inline const Address &getAddress() const { return addr; }
    inline uint64_t getLength() const { return len; }
    inline const std::string &getMNEM() const { return mnem; }
    inline const std::string &getBody() const { return body; }
};

struct Disassembly {
    std::vector<DisassemblyInstruction> instructions;

public:
    inline std::vector<DisassemblyInstruction> const &getInstructions() const { return instructions; }
};
class SleighBridge { private:
    // sleigh internals
    unique_ptr<Sleigh> sleigh;
    ContextInternal contextDb;
    DocumentStorage docStorage;
    Document *doc;
    Element *elt;

    // rust interop
    RustLoadImage loadImage;

public:
    SleighBridge(const std::string &path);

    std::unique_ptr<Disassembly>
    disassemble(const char *bytes, uint32_t len, uint64_t addr, uint32_t max_instructions);
};


class AssemblyEmitter : public AssemblyEmit {
public:
    AssemblyEmitter(DisassemblyInstruction &);

    DisassemblyInstruction &instruction;
    void dump(const Address &addr, const std::string &mnem, const std::string &body);
};

// ######################
// #  Bridge Functions  #
// ######################

std::unique_ptr<SleighBridge> create_sleigh_bridge(const std::string &path);

std::unique_ptr<Disassembly> SleighBridge_disassemble(
    SleighBridge &self,
    rust::Slice<const uint8_t> bytes,
    uint32_t len,
    uint64_t addr,
    uint32_t max_instructions
);

