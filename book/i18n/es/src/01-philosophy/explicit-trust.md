# 01.2 — Confianza explícita

> **Status:** Propuesto (PROPOSED)  
> **Summary:** NUDO separa los valores generados de los verificados para que las transiciones de confianza sean visibles para quienes revisan y para las herramientas de comprobación.

## La distinción central

```text
Generated<Article>
        ↓ verify
Verified<Article>
```

Un valor generado no es automáticamente un valor verificado, y el análisis
sintáctico no es la misma operación que la verificación.

Esto evita uno de los colapsos conceptuales más habituales en los sistemas cargados
de modelos: que «la salida tenía la forma correcta» se convierta en «se puede
confiar en la salida».

## Qué no es la verificación

No se pretende que sea un cast mágico. Un verificador puede fallar. La verificación
debería dejar una procedencia que explique qué entrada, qué regla, qué salida del
modelo y qué verificador produjeron el valor de confianza.
