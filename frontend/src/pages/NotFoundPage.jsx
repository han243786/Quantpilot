import { navigateTo, strategiesPath } from "../router";
import { useI18n } from "../i18n";

export default function NotFoundPage({ pathname = "" }) {
  const { t } = useI18n();

  return (
    <section className="not-found-page" aria-labelledby="not-found-title">
      <div className="not-found-page__mark" aria-hidden="true">
        404
      </div>
      <h1 id="not-found-title">{t("没有找到这个页面")}</h1>
      <p>
        {t("当前路径不存在或已迁移。你可以返回策略中心继续工作。")}
        {pathname ? <span className="not-found-page__path">{pathname}</span> : null}
      </p>
      <button className="ad-btn ad-btn--primary" type="button" onClick={() => navigateTo(strategiesPath())}>
        {t("返回策略中心")}
      </button>
    </section>
  );
}
