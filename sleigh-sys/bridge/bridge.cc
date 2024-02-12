#include "bridge.hh"
//#include "../target/cxxbridge/sleigh-sys/src/lib.rs.h"
#include "sleigh-sys/src/lib.rs.h"
#include <mutex>
#include <iostream>

unique_ptr<Decompiler> newDecompiler(RustLoadImage *loadImage,
                                     unique_ptr<DocumentStorage> spec) {
  auto l = unique_ptr<LoadImage>(new RustLoadImageProxy(loadImage));
  return make_unique<Decompiler>(move(l), move(spec));
}

unique_ptr<Address> newAddress() { return make_unique<Address>(); }

uint32_t getAddrSpaceType(const AddrSpace &space) {
  return (uint32_t)space.getType();
}

unique_ptr<Address> getVarnodeDataAddress(const VarnodeData &data) {
  return make_unique<Address>(data.getAddr());
}

AddrSpace *getVarnodeSpace(const VarnodeData &data) {
  return data.space;
}

uint64_t getVarnodeOffset(const VarnodeData &data) { return data.offset; }

uint64_t getVarnode_sizeof() {
    return sizeof(VarnodeData);
}

unique_ptr<ContextDatabase> newContext() {
  return unique_ptr<ContextDatabase>(new ContextInternal());
}

unique_ptr<DocumentStorage> newDocumentStorage(const std::string &s) {
  static std::mutex lock;
  std::lock_guard<std::mutex> guard(lock);

  auto doc = make_unique<DocumentStorage>();
  std::stringstream ss;
  ss << s;
  auto root = doc->parseDocument(ss)->getRoot();
  doc->registerTag(root);
  return doc;
}

void RustLoadImageProxy::loadFill(uint1 *ptr, int4 size,
                                  const Address &address) {
  return inner->load_fill(ptr, size, address);
}

void RustLoadImageProxy::adjustVma(long adjust) {
  return inner->adjust_vma(adjust);
}

void RustPCodeEmitProxy::dump(const Address &addr, OpCode opc,
                              VarnodeData *outvar, VarnodeData *vars,
                              int4 isize) {
  std::vector<VarnodeData> v;
  for (int i = 0; i < isize; i++) {
      v.push_back(vars[i]);
  }
  inner->dump(addr, (uint32_t)opc, outvar, v);
}

int32_t Decompiler::translate(RustPCodeEmit *emit, uint64_t addr, uint64_t limit) const {
  auto p = RustPCodeEmitProxy(emit);

  uint32_t off = 0;
  while (limit == 0 || off < limit) {
      auto address = Address(this->getDefaultCodeSpace(), addr + off);

      try {
         //std::cout << "translation: " << off << std::endl;
        off += this->oneInstruction(p, address);
      } catch (BadDataError &err) {
        break;
      } catch (UnimplError &err) {
        break;
      }
  }
  return off;
}

int32_t Decompiler::disassemble(RustAssemblyEmit *emit, uint64_t addr, uint64_t limit) const {
  auto p = RustAssemblyEmitProxy(emit);

  uint32_t off = 0;
  while (limit == 0 || off < limit) {
    auto address = Address(this->getDefaultCodeSpace(), addr + off);
    try {
      off += this->printAssembly(p, address);
    } catch (BadDataError &err) {
      break;
    }
  }

  return off;
}

uint32_t getVarnodeSize(const VarnodeData &data) { return data.size; }

void RustAssemblyEmitProxy::dump(const Address &addr, const string &mnem,
                                 const string &body) {
  this->inner->dump(addr, mnem, body);
}

void Decompiler::getRegisterList(std::vector<RegisterPair> &out) const {
    std::map<VarnodeData, std::string> reglist;
    getAllRegisters(reglist);
    for (const auto &entry : reglist) {
        RegisterPair pair{};
        pair.varnode = entry.first;
        pair.key = entry.second;
        out.push_back(std::move(pair));
    }
}
