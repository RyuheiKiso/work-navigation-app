// 自動生成（scripts/build-tokens.sh フォールバック）— 編集禁止
// 対応 §: ロードマップ §9.5.1
// 入力: docs/02_設計/design-tokens/

export const TOKENS = {
  color:
{
  "$schema": "https://schemas.style-dictionary.com/v3/schema.json",
  "_meta": {
    "name": "color",
    "section": "ロードマップ §9.5.1",
    "rationale": "屋外輝度 1,000nits 下でも判別可能なコントラストを保証する 12 トーン構成（success／warning／danger／info × 3 強度）。色のみで情報を伝えない設計（§11.2.2 WCAG 2.2 SC 1.4.1）に従い、危険操作には形・テキストも併用する。"
  },
  "color": {
    "base": {
      "white": { "value": "#FFFFFF" },
      "black": { "value": "#000000" }
    },
    "neutral": {
      "0":   { "value": "#FFFFFF" },
      "50":  { "value": "#F8F9FA" },
      "100": { "value": "#E9ECEF" },
      "200": { "value": "#DEE2E6" },
      "300": { "value": "#CED4DA" },
      "400": { "value": "#ADB5BD" },
      "500": { "value": "#6C757D" },
      "600": { "value": "#495057" },
      "700": { "value": "#343A40" },
      "800": { "value": "#212529" },
      "900": { "value": "#0D1117" }
    },
    "semantic": {
      "success": {
        "subtle":  { "value": "#D4EDDA" },
        "default": { "value": "#28A745" },
        "strong":  { "value": "#155724" }
      },
      "warning": {
        "subtle":  { "value": "#FFF3CD" },
        "default": { "value": "#FFC107" },
        "strong":  { "value": "#856404" }
      },
      "danger": {
        "subtle":  { "value": "#F8D7DA" },
        "default": { "value": "#DC3545" },
        "strong":  { "value": "#721C24" }
      },
      "info": {
        "subtle":  { "value": "#D1ECF1" },
        "default": { "value": "#17A2B8" },
        "strong":  { "value": "#0C5460" }
      }
    },
    "andon": {
      "_meta": { "section": "ロードマップ §9.3.1 §17 Andon 5 段階" },
      "level1_normal":   { "value": "#28A745" },
      "level2_attention": { "value": "#17A2B8" },
      "level3_warning":  { "value": "#FFC107" },
      "level4_alert":    { "value": "#FD7E14" },
      "level5_critical": { "value": "#DC3545" }
    },
    "outdoor": {
      "_meta": { "section": "ロードマップ §6.3 屋外 1,000nits 環境向け強コントラスト版" },
      "success_high":  { "value": "#0F5132" },
      "warning_high":  { "value": "#664D03" },
      "danger_high":   { "value": "#58151C" }
    }
  }
}
  ,
  elevation:
{
  "$schema": "https://schemas.style-dictionary.com/v3/schema.json",
  "_meta": {
    "name": "elevation",
    "section": "ロードマップ §9.5.1",
    "rationale": "屋外でも視認可能な濃度補正を含む 5 段階。屋外輝度 1,000nits 下では影が薄くなるため、`outdoor` 系列で濃度を強める。"
  },
  "shadow": {
    "elevation": {
      "0": { "value": "none" },
      "1": { "value": "0 1px 2px rgba(13, 17, 23, 0.06), 0 1px 3px rgba(13, 17, 23, 0.10)" },
      "2": { "value": "0 4px 6px rgba(13, 17, 23, 0.07), 0 2px 4px rgba(13, 17, 23, 0.06)" },
      "3": { "value": "0 10px 15px rgba(13, 17, 23, 0.10), 0 4px 6px rgba(13, 17, 23, 0.05)" },
      "4": { "value": "0 20px 25px rgba(13, 17, 23, 0.10), 0 10px 10px rgba(13, 17, 23, 0.04)" }
    },
    "outdoor": {
      "_meta": { "note": "屋外 1,000nits 下では影が薄く見えるため強コントラスト版（§6.3）" },
      "1": { "value": "0 1px 3px rgba(13, 17, 23, 0.30), 0 1px 4px rgba(13, 17, 23, 0.40)" },
      "2": { "value": "0 4px 8px rgba(13, 17, 23, 0.30), 0 2px 6px rgba(13, 17, 23, 0.40)" }
    }
  }
}
  ,
  haptic:
{
  "$schema": "https://schemas.style-dictionary.com/v3/schema.json",
  "_meta": {
    "name": "haptic",
    "section": "ロードマップ §9.5.4 §11.2.3",
    "rationale": "Android／Windows のハプティック API 差を吸収する共通カテゴリ。騒音下では音の代替として優先される。端末がハプティック非搭載の場合はビジュアルフラッシュへフォールバックする（§11.2.3 二重化）。"
  },
  "haptic": {
    "light": {
      "value": {
        "intensity": "weak",
        "pattern_ms": [20],
        "purpose": "タッチ受領"
      }
    },
    "confirm": {
      "value": {
        "intensity": "medium",
        "pattern_ms": [50],
        "purpose": "ステップ完了"
      }
    },
    "warning": {
      "value": {
        "intensity": "strong",
        "pattern_ms": [50, 80, 50],
        "purpose": "警告"
      }
    },
    "danger": {
      "value": {
        "intensity": "strong",
        "pattern_ms": [80, 120, 80, 120, 80],
        "purpose": "危険操作前"
      }
    },
    "policy": {
      "fallback_visual_flash": { "value": true, "_meta": { "note": "ハプティック非搭載端末では §11.2.3 二重化に従い視覚で代替" } },
      "fallback_sound_when_silent": { "value": true, "_meta": { "note": "音がミュートのとき強度を上げてハプティックで補完（§9.5.4）" } }
    }
  }
}
  ,
  icon:
{
  "$schema": "https://schemas.style-dictionary.com/v3/schema.json",
  "_meta": {
    "name": "icon",
    "section": "ロードマップ §9.5.1 §11.2.2",
    "rationale": "48dp ベース／単色＋形で識別／§11.2.2 色覚多様性整合。重要情報を色のみで区別しない（WCAG 2.2 SC 1.4.1）。"
  },
  "icon": {
    "size": {
      "small":  { "value": "16px" },
      "medium": { "value": "24px" },
      "large":  { "value": "32px" },
      "touch":  { "value": "48dp", "_meta": { "note": "タッチターゲット内アイコン基準" } }
    },
    "stroke": {
      "thin":    { "value": "1px" },
      "regular": { "value": "1.5px" },
      "bold":    { "value": "2px" }
    },
    "semantic": {
      "alert":      { "value": "warning-triangle",     "_meta": { "shape": "triangle" } },
      "success":    { "value": "check-circle",         "_meta": { "shape": "circle" } },
      "danger":     { "value": "octagon-exclamation",  "_meta": { "shape": "octagon" } },
      "info":       { "value": "info-circle",          "_meta": { "shape": "circle" } },
      "andon":      { "value": "andon-tower",          "_meta": { "shape": "stack" } },
      "audit":      { "value": "shield-check",         "_meta": { "shape": "shield" } },
      "deadletter": { "value": "envelope-alert",       "_meta": { "shape": "envelope" } }
    },
    "policy": {
      "color_alone_prohibited": { "value": true, "_meta": { "note": "WCAG 2.2 SC 1.4.1／§11.2.2" } },
      "shape_required": { "value": true, "_meta": { "note": "形状で識別（色覚多様性 P/D/T 対応）" } },
      "label_paired": { "value": true, "_meta": { "note": "アイコン単独は採用不可。常にテキストラベルを伴う" } }
    }
  }
}
  ,
  motion:
{
  "$schema": "https://schemas.style-dictionary.com/v3/schema.json",
  "_meta": {
    "name": "motion",
    "section": "ロードマップ §9.5.1 §9.5.2 §21 注 2",
    "rationale": "100ms 以下は知覚同期、240ms 以上は離脱の境界（§21 注 2）。意味的モーションは 6 カテゴリ、装飾的な弾性アニメは現場では認知妨害のため不採用。prefers-reduced-motion 時は装飾モーションを 0ms 化し、情報モーションのみ維持。"
  },
  "motion": {
    "duration": {
      "instant":  { "value": "0ms" },
      "short":    { "value": "80ms" },
      "standard": { "value": "160ms" },
      "long":     { "value": "240ms" }
    },
    "ease": {
      "standard":   { "value": "cubic-bezier(0.4, 0.0, 0.2, 1)" },
      "decelerated": { "value": "cubic-bezier(0.0, 0.0, 0.2, 1)" },
      "accelerated": { "value": "cubic-bezier(0.4, 0.0, 1.0, 1.0)" },
      "emphasized":  { "value": "cubic-bezier(0.2, 0.0, 0.0, 1.0)", "_meta": { "note": "Material Design 3 emphasized" } }
    },
    "category": {
      "_meta": { "note": "意味的モーションの 6 カテゴリ（§9.5.2）" },
      "enter":      { "value": { "duration": "{motion.duration.short.value}",    "ease": "{motion.ease.decelerated.value}" } },
      "exit":       { "value": { "duration": "{motion.duration.short.value}",    "ease": "{motion.ease.accelerated.value}" } },
      "transition": { "value": { "duration": "{motion.duration.standard.value}", "ease": "{motion.ease.emphasized.value}" } },
      "stateChange": { "value": { "duration": "{motion.duration.short.value}",   "ease": "{motion.ease.standard.value}" } },
      "success":    { "value": { "duration": "{motion.duration.standard.value}", "ease": "{motion.ease.decelerated.value}" } },
      "fail":       { "value": { "duration": "{motion.duration.long.value}",     "ease": "{motion.ease.standard.value}" } }
    },
    "reducedMotion": {
      "_meta": { "note": "prefers-reduced-motion: reduce のとき装飾モーションを 0ms 化（§9.5.2）" },
      "decoration": { "value": "0ms" },
      "information": { "value": "{motion.duration.short.value}" }
    }
  }
}
  ,
  radius:
{
  "$schema": "https://schemas.style-dictionary.com/v3/schema.json",
  "_meta": {
    "name": "radius",
    "section": "ロードマップ §9.5.1",
    "rationale": "0／4／8／16px の 4 段階。重要操作は 8px、危険操作は 0px（剛性表現で誤タップを抑止）。装飾的な大きい角丸は採用しない。"
  },
  "radius": {
    "surface": {
      "none":   { "value": "0px",  "_meta": { "note": "危険操作・確定操作（§9.2.2 誤操作予防）" } },
      "small":  { "value": "4px",  "_meta": { "note": "通常コンテナ" } },
      "medium": { "value": "8px",  "_meta": { "note": "重要操作（CTA／保存／公開）" } },
      "large":  { "value": "16px", "_meta": { "note": "モーダル・カード上段" } }
    },
    "pill": {
      "value": "9999px",
      "_meta": { "note": "ステータスチップ等の楕円形バッジ用" }
    }
  }
}
  ,
  sound:
{
  "$schema": "https://schemas.style-dictionary.com/v3/schema.json",
  "_meta": {
    "name": "sound",
    "section": "ロードマップ §9.5.3",
    "rationale": "製造現場の典型環境音 80〜95dB を上回らずに知覚可能な音設計。WHO 推奨 85dB 8 時間曝露上限を踏まえ、本アプリの通知音は設計時に 85dB 以下を保証する。すべて視覚等価物併設、BGM・ジングル不採用（§9.2.4 ダーク・パターン排除）。"
  },
  "sound": {
    "feedback": {
      "success": {
        "value": {
          "waveform": "tone",
          "notes": ["C5", "E5"],
          "duration_ms": 200,
          "amplitude_db": 70,
          "purpose": "ステップ完了"
        }
      },
      "fail": {
        "value": {
          "waveform": "tone",
          "notes": ["E4", "C4"],
          "duration_ms": 300,
          "amplitude_db": 70,
          "purpose": "完了条件不適合"
        }
      },
      "warning": {
        "value": {
          "waveform": "tone",
          "notes": ["A4", "A4", "A4"],
          "duration_ms": 500,
          "amplitude_db": 80,
          "purpose": "アンドン"
        }
      },
      "input": {
        "value": {
          "waveform": "tone",
          "notes": ["C6"],
          "duration_ms": 80,
          "amplitude_db": 60,
          "purpose": "タッチ・スキャン受領"
        }
      },
      "danger": {
        "value": {
          "waveform": "square",
          "frequency_hz": 350,
          "pattern": "interrupted",
          "duration_ms": 1000,
          "amplitude_db": 85,
          "purpose": "危険操作の確認段階"
        }
      }
    },
    "policy": {
      "_meta": { "note": "ミュート時は警告アイコンを常時表示（§9.5.3）" },
      "max_amplitude_db": { "value": 85 },
      "default_muted": { "value": false },
      "high_freq_attenuation_supported": { "value": true, "_meta": { "note": "難聴対応の可聴域シフト" } },
      "vibration_fallback_supported": { "value": true }
    }
  }
}
  ,
  spacing:
{
  "$schema": "https://schemas.style-dictionary.com/v3/schema.json",
  "_meta": {
    "name": "spacing",
    "section": "ロードマップ §9.5.1 §11.2 §21 注 6",
    "rationale": "4px ベース 8 段階。タッチターゲット最小 9mm（§21 注 6）／推奨 11mm を強制するため、`touchTarget.minimum` と `touchTarget.recommended` を別トークンとして固定。"
  },
  "space": {
    "unit": {
      "0":  { "value": "0px" },
      "1":  { "value": "4px" },
      "2":  { "value": "8px" },
      "3":  { "value": "12px" },
      "4":  { "value": "16px" },
      "5":  { "value": "24px" },
      "6":  { "value": "32px" },
      "7":  { "value": "48px" },
      "8":  { "value": "64px" }
    },
    "touchTarget": {
      "minimum":     { "value": "9mm",  "_meta": { "note": "ISO 9241-411／§21 注 6" } },
      "recommended": { "value": "11mm", "_meta": { "note": "Apple HIG 44pt ≈ 11mm／手袋操作想定" } }
    },
    "container": {
      "compact":  { "value": "320px" },
      "regular":  { "value": "768px" },
      "wide":     { "value": "1024px" },
      "full":     { "value": "100%" }
    }
  }
}
  ,
  typography:
{
  "$schema": "https://schemas.style-dictionary.com/v3/schema.json",
  "_meta": {
    "name": "typography",
    "section": "ロードマップ §9.5.1 §11.2.2 §11.3.1",
    "rationale": "Dynamic Type 連動で 14／16／20／24／32px の 5 段階を提供。CJK／ハングル／ラテンの同高保証はフォント連鎖（fallback chain）で担保し、§11.3 国際化に追従する。"
  },
  "font": {
    "family": {
      "sans": {
        "value": ["Inter", "Noto Sans JP", "Noto Sans KR", "Noto Sans SC", "Noto Sans TC", "Noto Sans Arabic", "Noto Sans Hebrew", "system-ui", "sans-serif"]
      },
      "mono": {
        "value": ["JetBrains Mono", "Source Han Code JP", "monospace"]
      },
      "industrial": {
        "_meta": { "note": "工業記号（°C・Ω・ねじ等、§11.2.3）。一般フォント未対応時のフォールバック" },
        "value": ["Noto Sans Symbols 2", "DejaVu Sans", "sans-serif"]
      }
    },
    "size": {
      "caption":  { "value": "12px" },
      "body":     { "value": "16px" },
      "subtitle": { "value": "20px" },
      "title":    { "value": "24px" },
      "display":  { "value": "32px" }
    },
    "lineHeight": {
      "tight":   { "value": "1.2" },
      "snug":    { "value": "1.4", "_meta": { "note": "日本語見出し向け（24/32px）。tight 1.2 では下端が窮屈、normal 1.5 では冗長になる中間" } },
      "normal":  { "value": "1.5" },
      "relaxed": { "value": "1.6", "_meta": { "note": "日本語本文向け。長文の可読性を上げる" } },
      "loose":   { "value": "1.75" }
    },
    "weight": {
      "regular":  { "value": 400 },
      "medium":   { "value": 500 },
      "semibold": { "value": 600, "_meta": { "note": "見出しの強調と CTA を区別するための中間ウェイト。bold は andon／display 専用に温存" } },
      "bold":     { "value": 700 }
    },
    "letterSpacing": {
      "tight":  { "value": "-0.01em" },
      "normal": { "value": "0" },
      "wide":   { "value": "0.05em" }
    }
  }
}
  ,
} as const;
