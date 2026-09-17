# 03.4 — Los diagnósticos como diseño de producto

> **Status:** Especificado / Planificado  
> **Summary:** Los errores del compilador deberían explicar la regla, la ubicación en el código y la cadena que introdujo el requisito.

Los diagnósticos forman parte de la experiencia del lenguaje, no son un volcado de
depuración.

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
