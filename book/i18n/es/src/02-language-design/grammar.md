# 02.1 — Disciplina de gramática y sintaxis

> **Status:** Implementado / Propuesto (IMPLEMENTED / PROPOSED)  
> **Summary:** La gramática es un contrato entre el texto fuente y el parser, implementado por el parser y comprobado por un verificador; la sintaxis provisional de agentes debe seguir siendo visiblemente provisional hasta que se acepte.

## Poca sintaxis, semántica fuerte

El objetivo de diseño de NUDO no es inventar una palabra clave para cada concepto de
IA. La sintaxis es cara, porque cada forma nueva afecta al análisis, a las
herramientas, al formateo, al aprendizaje y a la compatibilidad.

## Presión sobre el parser

Un parser de descenso recursivo funciona mejor cuando la gramática de expresiones
explicita la precedencia y evita la recursión por la izquierda. Una jerarquía de
expresiones robusta debería estructurarse conceptualmente así:

```text
logical-or
  → logical-and
  → equality
  → comparison
  → additive
  → multiplicative
  → unary
  → postfix
  → primary
```

El análisis postfijo es donde las llamadas, el acceso a campos y la indexación se
componen de forma natural.

## Palabras contextuales frente a palabras reservadas

Solo las palabras que realmente se aceptan como tokens reservados deberían tratarse
como reservadas por el lexer. La sintaxis futura, como `agent`, `task`, `ask` y
`verify`, necesita una decisión a nivel de NEP antes de que las herramientas
dependan de ella.
