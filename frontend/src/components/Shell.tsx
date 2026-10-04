import { useTranslation } from "react-i18next";

const navKeys = [
  "dashboard",
  "journal",
  "tradeList",
  "fields",
  "plugins",
  "backup",
  "settings",
] as const;

export default function Shell() {
  const { t } = useTranslation("translation");

  return (
    <div className="flex h-full min-h-screen bg-gray-50 text-gray-900" dir="rtl">
      <aside className="flex w-64 shrink-0 flex-col border-e border-gray-200 bg-white">
        <header className="border-b border-gray-200 px-4 py-4">
          <h1 className="text-lg font-bold">{t("app.title")}</h1>
          <p className="text-xs text-gray-500">{t("app.subtitle")}</p>
        </header>
        <nav className="flex-1 space-y-1 px-3 py-4" aria-label={t("nav.ariaLabel")}>
          {navKeys.map((key) => (
            <a
              key={key}
              href={`#${key}`}
              className="block rounded-md px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 hover:text-gray-900"
            >
              {t(`nav.${key}`)}
            </a>
          ))}
        </nav>
        <footer className="border-t border-gray-200 px-4 py-3 text-xs text-gray-500">
          {t("shell.offline")} • {t("shell.rtl")}
        </footer>
      </aside>
      <main className="min-w-0 flex-1 p-6">
        <h2 className="text-xl font-semibold">{t("shell.welcome")}</h2>
        <p className="mt-2 text-sm text-gray-600">{t("shell.status")}</p>
      </main>
    </div>
  );
}
