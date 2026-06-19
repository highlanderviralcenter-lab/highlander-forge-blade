# ADR 006: Traits para Injecao de Dependencia

## Status
Aceito

## Contexto
Codigo Windows-specific (WMI, Registry, Servicos) nao roda em CI Linux. Precisamos testar logica de negocio sem depender da plataforma.

## Decisao
#[cfg_attr(test, mockall::automock)] em todos os traits de provedor. CI Linux roda com mocks; CI Windows (semanal) roda integracao.

## Traits
- SystemInfoProvider -> WMI
- RegistryProvider -> Registry
- ServiceProvider -> SCManager
- UpdateProvider -> WUA
