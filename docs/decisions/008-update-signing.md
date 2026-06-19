# ADR-008: Assinatura de Updates com Ed25519

## Geracao do Par de Chaves

```bash
# 1. Gera chave privada (NAO commitar!)
openssl genpkey -algorithm ed25519 -out update_privkey.pem

# 2. Extrai chave publica em formato DER
openssl pkey -in update_privkey.pem -pubout -outform DER -out assets/update_pubkey.bin

# 3. Verifica (deve ter 32 bytes)
ls -la assets/update_pubkey.bin
```

## Seguranca
- NUNCA commitar `update_privkey.pem`
- Chave privada em HSM ou secret manager
- `assets/update_pubkey.bin` pode ser commitado (eh publica)

## Rotacao
1. Gerar novo par
2. Substituir `assets/update_pubkey.bin`
3. Novo release com chave embutida
4. Versoes antigas nao reconhecem updates com nova chave
