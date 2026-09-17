# 02.2 — Expresiones y precedencia

> **Estado:** Especificado (SPECIFIED)  
> **Resumen:** La precedencia, la asociatividad y la prohibición de recursión por la izquierda están decididas y se comprueban mecánicamente. El parser que las implementa es M2.

La precedencia no es un detalle de formato. `a + b * c` debe tener un único
significado en toda implementación conforme.

Ya no son preguntas abiertas. Están enunciadas una sola vez, como una cadena de
niveles gramaticales, en
[`grammar/nudo.ebnf`](https://github.com/smouj/nudo/blob/main/grammar/nudo.ebnf),
copiadas de forma idéntica en los dos documentos que las describen y comprobadas
mecánicamente por
[`scripts/check-grammar.py`](https://github.com/smouj/nudo/blob/main/scripts/check-grammar.py):

- las operaciones postfijas ligan más fuerte que los operadores unarios, y encadenan,
  de modo que `a.b(c)[d]` es una sola expresión;
- la multiplicación y la división ligan más fuerte que la suma y la resta, ambas
  asociativas por la izquierda;
- la comparación **no** encadena: `a < b < c` es un error de sintaxis y no
  `(a < b) < c`, y las comparaciones se combinan con `&&`;
- los operadores booleanos tienen precedencia fija, con `||` como el más laxo;
- no hay nivel de asignación, porque está sin decidir si NUDO tiene siquiera
  enlaces mutables. Escribir ese nivel ahora sería inventar una característica del
  lenguaje para rellenar un hueco en un diagrama;
- `ask`, `verify` y `delegate` son expresiones primarias, no operadores, que es lo
  que hace analizable `verify draft with V`: `with` no es un operador y ninguna
  forma postfija empieza por él.

La regla que hace que esto sea comprobable en lugar de aspiracional es **no hay
recursión por la izquierda**. Un parser descendente recursivo no puede tratar una
producción que derive una forma que empiece por sí misma, y la primera revisión de
esta gramática tenía cuatro. Ahora es un fallo de build, no un descubrimiento
pendiente.

El formateador debería imprimir sintaxis suficiente para preservar el significado
sin depender de la intuición humana.
