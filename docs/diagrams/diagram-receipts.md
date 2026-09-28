# Architecture diagram delivery evidence

Both diagrams are conceptual proposed architectures. Arrows show selected command/data relationships; reverse events and detailed protocol handshakes are explained in the report. All authored edges have semantic labels. Validation here concerns diagram artifacts, not the security or correctness of a built product.


**Overall architecture**

- Diagram type: `architecture`
- HTML: [overall.html](overall.html)
- Specification: [overall.architecture.json](overall.architecture.json)
- Specification SHA-256: `5646758cda8ce8bcd68949d30a592d7a980a078e72716efc815e7fd89d9c9a55` (4,936 bytes)
- Artifact SHA-256: `bfa3b6263d01db56b049beb67ca87bc2eafb11915e96d95420a21862e25e6dd9` (811,391 bytes)
- Validation: **9/9 showcase; 0 errors, 0 warnings**.
- Browser evidence: **passed**, using the delivered HTML without modification.
- Perceptual visual review: **passed**, image-capable review of the 1440×900 dark and 2048×1320 light captures. No crossing/overlap/clipping or conspicuous lower empty band observed.
- Post-delivery visual correction rounds: **0**.
- Full receipts: [delivery](overall.delivery.json), [automated browser](overall.visual-check.json), [capture contact sheet](overall.visual-check.html).


| Viewport | Measured page | Horizontal overflow | Vertical overflow |
|---|---|---|---|

| 1440×900 | 1440×900 | No | No |

| 1600×1000 | 1600×1000 | No | No |

| 1920×1080 | 1920×1080 | No | No |

| 2048×1320 | 2048×1320 | No | No |

**Execution architecture**

- Diagram type: `architecture`
- HTML: [execution.html](execution.html)
- Specification: [execution.architecture.json](execution.architecture.json)
- Specification SHA-256: `2cdb20d659b0be90a27964e0bea250972900ac6184565fc5e108585b42ac6de8` (4,567 bytes)
- Artifact SHA-256: `2f8df485bd1cc8ac439fe0883276c1949ac06d3786324094d97d1b3e4d757bc0` (809,432 bytes)
- Validation: **9/9 showcase; 0 errors, 0 warnings**.
- Browser evidence: **passed**, using the delivered HTML without modification.
- Perceptual visual review: **passed**, image-capable review of the 1440×900 dark and 2048×1320 light captures. No crossing/overlap/clipping or conspicuous lower empty band observed.
- Post-delivery visual correction rounds: **0**.
- Full receipts: [delivery](execution.delivery.json), [automated browser](execution.visual-check.json), [capture contact sheet](execution.visual-check.html).


| Viewport | Measured page | Horizontal overflow | Vertical overflow |
|---|---|---|---|

| 1440×900 | 1440×900 | No | No |

| 1600×1000 | 1600×1000 | No | No |

| 1920×1080 | 1920×1080 | No | No |

| 2048×1320 | 2048×1320 | No | No |


Supplementary interaction check: the overall viewer opened in the Codex browser; node search narrowed to Rust host daemon; selecting the node showed its incoming/outgoing relationships; closing the semantic passport restored the diagram; export options opened. Exported file contents and every optional viewer feature were not exercised. The automated receipts correctly retain `visualReview: pending`; perceptual review is recorded separately here.
