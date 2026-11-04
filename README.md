<a id="top"></a>

<div align="center">

<img src="branding/png/fahras-128.png" alt="Fahras logo" width="112">

# Fahras · فهرس

**A fast, keyboard-first file manager for Windows and Linux.**<br>
Instant search, rich previews, sandboxed security, and full Arabic support.

[![License: MIT](https://img.shields.io/badge/license-MIT-CFA24A)](LICENSE)
[![Platforms](https://img.shields.io/badge/platforms-Windows%2010%20%7C%2011%20%C2%B7%20Linux-25695B)](#download)
[![Status](https://img.shields.io/badge/status-early%20development-D4644A)](#roadmap)

[**العربية ⬇**](#arabic) · [Why Fahras](#why-fahras) · [Download](#download) · [Features](#features) · [Shortcuts](#keyboard-shortcuts) · [Roadmap](#roadmap)

</div>

> [!NOTE]
> **Fahras is in early development.** The Windows version is being built right now and there is no download yet.
> Star or watch the repository to follow along.

<div align="center">
<img src="docs/assets/strata-columns.png" alt="Columns, Icons and List views with search, a context menu, file properties and settings" width="900">
<br><sub>Current interface on Linux. The Windows look and the Explorer-style layouts are on the way.</sub>
</div>

## Why Fahras?

*Fahras* (فهرس) is Arabic for **index**: the list that helps you find anything fast.

- ⚡ **Fast with huge folders.** A folder of 100,000 files loads in under two seconds on Linux, and the window never freezes.
- 🔍 **Instant search.** Fuzzy filename search across all your drives, with results as you type.
- 🛡️ **Safe previews.** Images, PDFs and videos are opened in an isolated sandbox, so a malicious file can't reach your system just by being previewed.
- ⌨️ **Keyboard-first.** Every action has a shortcut. The mouse is welcome but never required.
- 👁️ **Rich previews.** Markdown, code, PDFs, camera RAW photos, video storyboards and audio with a live spectrum.
- 🌍 **Arabic built in.** A full Arabic interface with proper right-to-left layout, plus 13 other languages.
- 🎨 **Your style.** 95 themes, light and dark, and (coming soon) Windows 10 and 11 Explorer-style layouts.

## Download

| Platform | Status |
|---|---|
| 🪟 **Windows 10 / 11** (x64) | 🚧 In development, no release yet |
| 🐧 **Linux** | 🚧 First Fahras release coming soon. Until then you can [build from source](docs/packaging.md). |

## Features

### Browsing
- **Three views:** Columns (like macOS Finder), Icons grid, and a sortable List.
- **Tabs, history and pins** for your favorite places.
- **Never freezes:** folders load in the background and only what's visible gets drawn.

### Search
- **Ctrl+K** searches your home folder and every mounted drive by name or path.
- **Ctrl+F** filters the current folder as you type.
- Results start appearing while the search is still running.

### Previews
- Press **Space** to preview almost anything: images, camera RAW, PDF, Markdown, HTML, source code, audio and video.
- **File Properties** shows media details: resolution, duration, codecs, HDR, camera, lens and GPS location.

### File operations
- Copy, move, rename, trash, permanent delete and new folders, all cancellable with progress.
- Browse inside ZIP, 7z, TAR and RAR archives.
- Drop images, text or links from a browser to save them as files.

### Power tools
- **Custom actions:** add your own scripts to the right-click menu.
- **Network locations:** SMB shares and SFTP servers.
- **Custom shortcuts** for every action, plus an optional Vim-style mode.

## Fahras vs. File Explorer

| | File Explorer | Fahras |
|---|:---:|:---:|
| Opens huge folders without freezing | ⚠️ | ✅ |
| Instant search without setting up indexing | ❌ | ✅ |
| Column view | ❌ | ✅ |
| Sandboxed file previews | ❌ | ✅ |
| Markdown, code, RAW and video previews | ⚠️ Limited | ✅ |
| Full keyboard control and custom shortcuts | ⚠️ | ✅ |
| Custom right-click actions without registry edits | ❌ | ✅ |
| Themes | ❌ | ✅ 95 |
| Built into Windows, opens instantly | ✅ | ❌ |
| Open source | ❌ | ✅ |

## Keyboard shortcuts

| Action | Keys |
|---|---|
| Search everywhere | `Ctrl` `K` |
| Filter this folder | `Ctrl` `F` |
| Preview the selected file | `Space` |
| Rename | `F2` |
| Go to a path | `Ctrl` `L` |
| Back / Forward / Up one level | `Alt` `←` / `Alt` `→` / `Alt` `↑` |
| New tab / close tab | `Ctrl` `T` / `Ctrl` `W` |
| Refresh | `F5` |
| Settings | `Ctrl` `,` |
| **All shortcuts** | `F1` |

## Roadmap

**Phase 1: Foundation**
- [x] Fahras name, logo and repository
- [ ] Windows build on GitHub Actions
- [ ] Opens and browses folders on Windows
- [ ] Full Arabic translation with right-to-left layout

**Phase 2: Everyday use on Windows**
- [ ] Recycle Bin, drives, USB eject
- [ ] Ultra-fast whole-drive search using the NTFS file table
- [ ] Windows thumbnails

**Phase 3: Previews and security**
- [ ] Sandboxed previews on Windows

**Phase 4: Integration**
- [ ] Network shares, "Open in Fahras" in Explorer, run as administrator

**Phase 5: Release**
- [ ] Installer, signed updates, first public release

**Planned features**
- [ ] Windows 10 and Windows 11 Explorer-style layouts and skins
- [ ] Choose your app icon: Manila, Forest, Ink or Clay
- [ ] Folder sizes in the list
- [ ] Dual-pane mode
- [ ] Batch rename with live preview
- [ ] Hijri dates alongside Gregorian
- [ ] Smart "Recent" view and colored favorites
- [ ] Duplicate file finder

## Credits

Fahras is built on [**Strata**](https://github.com/lgse/strata) by **LGSE Ltd.**, an excellent file manager for Linux. Thank you to its authors and contributors.
Fahras is an independent project and is not affiliated with or endorsed by LGSE Ltd.

Third-party components and their licenses are listed in [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).

## License

[MIT](LICENSE). Copyright © 2026 LGSE Ltd. and © 2026 iBar8.

<br>

---

<a id="arabic"></a>

<div dir="rtl" align="right">

<div align="center">

<img src="branding/png/fahras-128.png" alt="شعار فهرس" width="96">

# فهرس · Fahras

**مدير ملفات سريع لنظامَي ويندوز ولينكس، يمكنك التحكم فيه بالكامل من لوحة المفاتيح.**<br>
بحث فوري، ومعاينة غنية للملفات، وحماية بعزل الملفات، ودعم كامل للغة العربية.

</div>

> 📌 **فهرس في مرحلة التطوير الأولى.** نعمل الآن على نسخة ويندوز، ولا يوجد إصدار للتنزيل بعد.
> تابع المستودع ليصلك كل جديد.

## لماذا فهرس؟

**الفهرس** هو القائمة التي تساعدك على الوصول إلى أي شيء بسرعة، وهذا ما صُمم البرنامج من أجله.

- ⚡ **سريع مع المجلدات الضخمة.** يفتح مجلدًا فيه مئة ألف ملف في أقل من ثانيتين على لينكس، دون أن تتجمد النافذة.
- 🔍 **بحث فوري.** ابحث عن الملفات بالاسم في جميع أقراصك، وتظهر النتائج أثناء الكتابة.
- 🛡️ **معاينة آمنة.** تُفتح الصور وملفات PDF والفيديو داخل بيئة معزولة، فلا يستطيع ملف ضار أن يصل إلى نظامك لمجرد معاينته.
- ⌨️ **لوحة المفاتيح أولًا.** لكل أمر اختصار، ويمكنك استخدام الفأرة متى شئت.
- 👁️ **معاينة غنية.** ملفات Markdown، والشيفرة البرمجية، وPDF، وصور الكاميرا الخام (RAW)، والفيديو، والصوت.
- 🌍 **العربية جزء أساسي.** واجهة عربية كاملة من اليمين إلى اليسار، إلى جانب ثلاث عشرة لغة أخرى.
- 🎨 **بأسلوبك.** خمسة وتسعون مظهرًا بألوان فاتحة وداكنة، وقريبًا تصاميم تشبه مستكشف الملفات في ويندوز 10 و11.

## التنزيل

| النظام | الحالة |
|---|---|
| 🪟 **ويندوز 10 و11** (x64) | 🚧 قيد التطوير، لا يوجد إصدار بعد |
| 🐧 **لينكس** | 🚧 أول إصدار من فهرس قريبًا. يمكنك حتى ذلك الحين [بناؤه من الشيفرة المصدرية](docs/packaging.md). |

## المزايا

### التصفح
- **ثلاث طرق للعرض:** الأعمدة، والأيقونات، والقائمة القابلة للترتيب.
- **علامات تبويب، وسجل للتنقل، ومواضع مثبتة** للأماكن التي تزورها كثيرًا.
- **لا يتجمد أبدًا:** تُحمَّل المجلدات في الخلفية، ولا يُرسم إلا ما يظهر على الشاشة.

### البحث
- اضغط **Ctrl+K** للبحث في مجلدك الشخصي وجميع الأقراص بالاسم أو المسار.
- اضغط **Ctrl+F** لتصفية محتوى المجلد الحالي أثناء الكتابة.
- تبدأ النتائج بالظهور قبل أن ينتهي البحث.

### المعاينة
- اضغط **مسافة** لمعاينة أغلب الملفات: الصور، وصور الكاميرا الخام، وPDF، وMarkdown، وHTML، والشيفرة البرمجية، والصوت، والفيديو.
- **خصائص الملف** تعرض تفاصيل الوسائط: الدقة، والمدة، والترميز، وHDR، ونوع الكاميرا والعدسة، والموقع الجغرافي.

### العمليات على الملفات
- النسخ، والنقل، وإعادة التسمية، والنقل إلى سلة المحذوفات، والحذف النهائي، وإنشاء المجلدات، مع إمكانية الإلغاء وعرض التقدم.
- تصفح محتوى ملفات ZIP و7z وTAR وRAR دون فكها.
- اسحب الصور أو النصوص أو الروابط من المتصفح لحفظها ملفاتٍ.

### أدوات متقدمة
- **أوامر مخصصة:** أضف برامجك النصية إلى قائمة الزر الأيمن.
- **مواقع الشبكة:** المجلدات المشتركة (SMB) وخوادم SFTP.
- **اختصارات قابلة للتعديل** لكل أمر، مع وضع اختياري بأسلوب Vim.

## فهرس مقارنةً بمستكشف الملفات

| | مستكشف الملفات | فهرس |
|---|:---:|:---:|
| فتح المجلدات الضخمة دون تجمد | ⚠️ | ✅ |
| بحث فوري دون إعداد الفهرسة | ❌ | ✅ |
| عرض الأعمدة | ❌ | ✅ |
| معاينة الملفات في بيئة معزولة | ❌ | ✅ |
| معاينة Markdown والشيفرة والصور الخام والفيديو | ⚠️ محدودة | ✅ |
| تحكم كامل من لوحة المفاتيح واختصارات مخصصة | ⚠️ | ✅ |
| أوامر مخصصة في قائمة الزر الأيمن دون تعديل السجل | ❌ | ✅ |
| المظاهر | ❌ | ✅ 95 |
| مدمج في ويندوز ويفتح فورًا | ✅ | ❌ |
| مفتوح المصدر | ❌ | ✅ |

## اختصارات لوحة المفاتيح

| الأمر | المفاتيح |
|---|---|
| البحث في كل مكان | <span dir="ltr">`Ctrl` `K`</span> |
| تصفية المجلد الحالي | <span dir="ltr">`Ctrl` `F`</span> |
| معاينة الملف المحدد | <span dir="ltr">`Space`</span> |
| إعادة التسمية | <span dir="ltr">`F2`</span> |
| الانتقال إلى مسار | <span dir="ltr">`Ctrl` `L`</span> |
| رجوع / تقدم / المجلد الأعلى | <span dir="ltr">`Alt` `←` / `Alt` `→` / `Alt` `↑`</span> |
| علامة تبويب جديدة / إغلاقها | <span dir="ltr">`Ctrl` `T` / `Ctrl` `W`</span> |
| تحديث | <span dir="ltr">`F5`</span> |
| الإعدادات | <span dir="ltr">`Ctrl` `,`</span> |
| **جميع الاختصارات** | <span dir="ltr">`F1`</span> |

## خطة العمل

**المرحلة الأولى: الأساس**
- [x] الاسم والشعار والمستودع
- [ ] بناء نسخة ويندوز تلقائيًا على GitHub
- [ ] فتح المجلدات وتصفحها على ويندوز
- [ ] ترجمة عربية كاملة مع واجهة من اليمين إلى اليسار

**المرحلة الثانية: الاستخدام اليومي على ويندوز**
- [ ] سلة المحذوفات، والأقراص، والإخراج الآمن لأجهزة USB
- [ ] بحث فائق السرعة في القرص كله عبر جدول ملفات NTFS
- [ ] الصور المصغرة من ويندوز

**المرحلة الثالثة: المعاينة والأمان**
- [ ] معاينة معزولة على ويندوز

**المرحلة الرابعة: التكامل مع النظام**
- [ ] مجلدات الشبكة، وأمر «فتح في فهرس» داخل مستكشف الملفات، والتشغيل بصلاحيات المسؤول

**المرحلة الخامسة: الإصدار**
- [ ] برنامج التثبيت، وتحديثات موقعة، وأول إصدار عام

**مزايا مخطط لها**
- [ ] تصاميم ومظاهر تشبه مستكشف الملفات في ويندوز 10 و11
- [ ] اختيار أيقونة البرنامج: مانيلا، أو الغابة، أو الحبر، أو الطين
- [ ] عرض أحجام المجلدات في القائمة
- [ ] وضع اللوحتين المتجاورتين
- [ ] إعادة تسمية مجموعة ملفات مع معاينة فورية
- [ ] عرض التاريخ الهجري بجانب الميلادي
- [ ] عرض ذكي للملفات الأخيرة، ومفضلة ملونة
- [ ] البحث عن الملفات المكررة

## شكر وتقدير

بُني فهرس على برنامج [**Strata**](https://github.com/lgse/strata) من شركة **LGSE Ltd.**، وهو مدير ملفات ممتاز لنظام لينكس. نشكر مطوريه وكل من ساهم فيه.
فهرس مشروع مستقل، ولا يتبع شركة LGSE Ltd. ولا يحظى بتأييدها.

## الترخيص

[رخصة MIT](LICENSE). حقوق النشر © 2026 LGSE Ltd. و© 2026 iBar8.

[⬆ العودة إلى الأعلى](#top)

</div>
