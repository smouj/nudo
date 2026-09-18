# 03.4 — Los diagnósticos como diseño de producto

> **Status:** Implementado / Planificado (IMPLEMENTED / PLANNED)  
> **Summary:** Los errores del compilador explican la regla, la ubicación en el código y la cadena que introdujo el requisito.

Los diagnósticos forman parte de la experiencia del lenguaje, no son un volcado de
depuración. Las familias léxica y sintáctica (`NDO1xxx`) están implementadas: un
archivo se rechaza con un código estable, un span exacto y un par esperado/encontrado.
Las familias de tipos, efectos y capacidades están planificadas, y la cadena que
tendrán que mostrar está especificada en
`spec/errors.md`.

Un error de capacidades de alta calidad debería identificar tanto el punto de la
llamada como el motivo:

```text
error[NDO3xxx]: missing capability `Network`

  ┌─ src/main.nudo:14:5
  │
14│     fetch(url)
  │     ^^^^^^^^^^ requires Network
  │
  └─ current task was not granted Network
```

Para los tipos de confianza, los errores deberían explicar que `Generated<T>` y
`Verified<T>` son distintos de forma intencionada, y señalar la frontera de
verificación explícita en lugar de recomendar un cast inseguro.
