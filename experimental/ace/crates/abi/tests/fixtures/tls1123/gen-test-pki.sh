#!/usr/bin/env bash
# PLAN-1123 T-01 probe: generate the loopback-only TLS test PKI with OpenSSL.
# TEST-ONLY material (127.0.0.1 peers, private test CAs) — never a real identity.
#   ca-test    : the LOCAL TEST TRUST (the CA the client is told to trust)
#   ca-untrust : an issuer the client is NOT told to trust
#   leaf-ok    : SAN localhost + 127.0.0.1, signed by ca-test   (must pass)
#   leaf-name  : SAN other.invalid,          signed by ca-test   (hostname fail)
#   leaf-ca    : SAN localhost + 127.0.0.1, signed by ca-untrust (trust fail)
set -euo pipefail
# Git Bash rewrites a leading "/CN=..." subject into a Windows path; keep argv verbatim.
export MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL='*'
out="${1:?usage: gen-test-pki.sh <out-dir>}"
mkdir -p "$out"
cd "$out"
days=10950   # 30 years: fixtures must not expire under the daily lane

mk() { # mk <name> <subject> <san> <issuer-ca>
  local name=$1 subj=$2 san=$3 issuer=$4
  openssl ecparam -name prime256v1 -genkey -noout -out "$name.key.pem" 2>/dev/null
  if [ "$issuer" = "-" ]; then
    openssl req -new -x509 -key "$name.key.pem" -out "$name.cert.pem" -days "$days" \
      -subj "$subj" -addext "basicConstraints=critical,CA:TRUE" \
      -addext "keyUsage=critical,keyCertSign,cRLSign" 2>/dev/null
  else
    openssl req -new -key "$name.key.pem" -out "$name.csr.pem" -subj "$subj" 2>/dev/null
    printf 'subjectAltName=%s\nbasicConstraints=CA:FALSE\nkeyUsage=digitalSignature\nextendedKeyUsage=serverAuth\n' "$san" > "$name.ext"
    openssl x509 -req -in "$name.csr.pem" -CA "$issuer.cert.pem" -CAkey "$issuer.key.pem" \
      -CAcreateserial -out "$name.cert.pem" -days "$days" -extfile "$name.ext" 2>/dev/null
    rm -f "$name.csr.pem" "$name.ext"
  fi
  openssl x509 -in "$name.cert.pem" -outform DER -out "$name.cert.der"
}

mk ca-test    "/CN=ACE TLS Test CA (loopback only)"          "-"                              "-"
mk ca-untrust "/CN=ACE TLS Untrusted Issuer (loopback only)" "-"                              "-"
mk leaf-ok    "/CN=localhost"  "DNS:localhost,IP:127.0.0.1"  ca-test
mk leaf-name  "/CN=other"      "DNS:other.invalid"           ca-test
mk leaf-ca    "/CN=localhost"  "DNS:localhost,IP:127.0.0.1"  ca-untrust
rm -f ./*.srl
# the server needs key+chain in one PEM
cat leaf-ok.key.pem   leaf-ok.cert.pem   > leaf-ok.server.pem
cat leaf-name.key.pem leaf-name.cert.pem > leaf-name.server.pem
cat leaf-ca.key.pem   leaf-ca.cert.pem   > leaf-ca.server.pem
ls -la
