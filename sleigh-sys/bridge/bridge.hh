#pragma once

#include <cstdint>
#include <memory>
#include <mutex>
#include <sstream>
#include <vector>
#include <map>

#include "../decompiler/address.hh"
#include "../decompiler/globalcontext.hh"
#include "../decompiler/loadimage.hh"
#include "../decompiler/opbehavior.hh"
#include "../decompiler/sleigh.hh"
#include "../decompiler/space.hh"

using std::make_unique;
using std::move;
using std::unique_ptr;

class RustPCodeEmit;

class RustPCodeEmitProxy : public PcodeEmit {
private:
  RustPCodeEmit *inner;

public:
  RustPCodeEmitProxy(RustPCodeEmit *emit) : inner(emit) {}

  virtual void dump(const Address &addr, OpCode opc, VarnodeData *outvar,
                    VarnodeData *vars, int4 isize);
};

class RustAssemblyEmit;
class RustAssemblyEmitProxy : public AssemblyEmit {
private:
  RustAssemblyEmit *inner;

public:
  RustAssemblyEmitProxy(RustAssemblyEmit *inner) : inner(inner) {}

  virtual void dump(const Address &addr, const string &mnem,
                    const string &body);
};

class RustLoadImage;

class RustLoadImageProxy : public LoadImage {
private:
  RustLoadImage *inner;

public:
  RustLoadImageProxy(RustLoadImage *inner)
      : LoadImage("nofile"), inner(inner) {}

  virtual void loadFill(uint1 *ptr, int4 size, const Address &address);
  virtual string getArchType(void) const { return "plain"; }
  virtual void adjustVma(long adjust);
};

class RegisterPair {
public:
    std::string key;
    VarnodeData varnode;

    const std::string &getKey() const { return this->key; }
    const VarnodeData &getVarnode() const { return this->varnode; }
};

class Decompiler : Sleigh {
private:
  unique_ptr<LoadImage> loadImage;
  unique_ptr<DocumentStorage> spec;
  ContextInternal context;

public:
  Decompiler(unique_ptr<LoadImage> loadImage, unique_ptr<DocumentStorage> spec)
      : Sleigh(loadImage.get(), &this->context), loadImage(move(loadImage)),
        spec(move(spec)) {
    this->initialize(*this->spec);
  }

  int32_t translate(RustPCodeEmit *emit, uint64_t addr, uint64_t limit) const;
  int32_t disassemble(RustAssemblyEmit *emit, uint64_t addr, uint64_t limit) const;
  ContextDatabase *getContext() { return &this->context; }
  void getRegisterList(std::vector<RegisterPair> &out) const;
};

unique_ptr<Decompiler> newDecompiler(RustLoadImage *loadImage,
                                     unique_ptr<DocumentStorage> spec);
unique_ptr<Address> newAddress();
unique_ptr<ContextDatabase> newContext();
unique_ptr<DocumentStorage> newDocumentStorage(const std::string &s);

uint32_t getAddrSpaceType(const AddrSpace &space);

uint32_t getVarnodeSize(const VarnodeData &data);
unique_ptr<Address> getVarnodeDataAddress(const VarnodeData &data);
AddrSpace *getVarnodeSpace(const VarnodeData &data);
uint64_t getVarnodeOffset(const VarnodeData &data);
uint64_t getVarnode_sizeof();