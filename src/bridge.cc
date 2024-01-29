#include "pcode/include/bridge.hh"

RustLoadImage::RustLoadImage() :
    LoadImage("RustLoadImage")
{
    this->base_address = 0;
    this->length = 0;
    this->bytes = NULL;
}

void RustLoadImage::setBytes(uintb base_address, const uint8_t *bytes,
        int4 len)
{
    this->base_address = base_address;
    this->bytes = bytes;
    this->length = len;
}

std::string RustLoadImage::getArchType(void) const { return "RustLoadImage::ArchType"; }

void RustLoadImage::adjustVma(long) {}

void RustLoadImage::loadFill(uint1 *dst, int4 size, const Address &addr)
{
    uintb offset = addr.getOffset();
    if (offset < this->base_address) {
        throw std::out_of_range("unable to load bytes outside of address range");
    }

    uintb range_boundary = this->base_address + this->length;
    if (offset >= range_boundary) {
        throw std::out_of_range("unable to load bytes past bounds of address range");
    }

    int4 i;
    for (i = 0; i < size; ++i)
    {
        uintb global_offset = base_address + i;
        if (global_offset < this->base_address || global_offset >= range_boundary) {
            dst[i] = 0;
        } else {
            uintb relative_offset = global_offset - this->base_address;
            dst[i] = this->bytes[relative_offset];
        }
    }
}


SleighBridge::SleighBridge(const std::string &path)
{
    doc = docStorage.openDocument(path);
    elt = doc->getRoot();
    docStorage.registerTag(elt);

    sleigh.reset(new Sleigh(&loadImage, &contextDb));
    sleigh->initialize(docStorage);
}


std::unique_ptr<Disassembly>
SleighBridge::disassemble(const char *bytes, uint32_t len, uint64_t address,
        uint32_t max_instructions)
{
    std::unique_ptr<Disassembly> out(new Disassembly);

    sleigh->reset(&loadImage, &contextDb);
    loadImage.setBytes(address, (const uint8_t *) bytes, len);


    uint32_t i;
    for (i = 0; i < len; ++i) {
        if (max_instructions && out->instructions.size() == max_instructions) {
            break;
        }

        Address addr(sleigh->getDefaultCodeSpace(), address + i);

        out->instructions.emplace_back();
        DisassemblyInstruction &instruction = out->instructions.back();
        
        try {
            AssemblyEmitter emitter{instruction};
            instruction.len = sleigh->printAssembly(emitter, addr);
        } catch (BadDataError &err) {
            if (i == 0) {
                break;
            }
            throw err;
        }
    }

    return out;
}

AssemblyEmitter::AssemblyEmitter(DisassemblyInstruction &instruction) : instruction{instruction} {}

void AssemblyEmitter::dump(const Address &addr, const std::string &mnem,
        const std::string &body)
{
    instruction.addr = addr;
    instruction.mnem = mnem;
    instruction.body = body;
}

std::unique_ptr<SleighBridge> create_sleigh_bridge(const std::string &path)
{
    return std::unique_ptr<SleighBridge>(new SleighBridge(path));
}
