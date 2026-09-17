# 01.1 — Primero el software determinista

> **Status:** Especificado (SPECIFIED)  
> **Summary:** El código determinista ordinario debe seguir siendo el caso fácil y predecible, aunque NUDO esté diseñado para sistemas de la era de la IA.

Un lenguaje para agentes que vuelve incómodo el software sencillo está resolviendo
el problema equivocado. Por eso NUDO trata la aritmética, el flujo de control, las
funciones, los datos y los módulos como el fundamento. Las construcciones agénticas
se apoyan sobre ese fundamento.

## Determinismo alrededor de la probabilidad

La forma prevista es:

```text
validated deterministic input
        ↓
probabilistic operation
        ↓
Generated<T>
        ↓
explicit validation / policy / approval
        ↓
deterministic continuation
```

Al modelo se le permite ser incierto. El programa no debe fingir que esa
incertidumbre desapareció solo porque una respuesta se haya analizado
sintácticamente con éxito.
