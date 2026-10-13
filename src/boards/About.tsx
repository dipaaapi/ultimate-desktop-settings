import { api } from '../lib/api';
import { TECH_STACK, TOOLING } from '../lib/stack';
import { useAsync } from '../lib/useAsync';
import { Badge, Row, Section } from '../components/ui';

export default function About() {
  const info = useAsync(api.appInfo);

  return (
    <>
      <Section title="Ultimate Windows Desktop Settings">
        <div className="list">
          <Row title="Version" description={info.data ? `${info.data.version} · ${info.data.platform}` : '…'} />
          <Row title="Developer" description="EdMaster28 · Software Developer & Computer Instructor · dipaaapi8@gmail.com" />
          <Row
            title="Built with Claude"
            description="Written by Claude, Anthropic's AI, through Claude Code, following EdMaster28's ideas, design and instructions."
          />
          {info.data && <Row title="Your data" description={<span className="mono">{info.data.dataDir}</span>} />}
        </div>
      </Section>

      <Section title="Tech stack">
        <p className="note" style={{ marginTop: 0, marginBottom: 10 }}>
          The tech stack is listed here.
        </p>
        <div className="list">
          {TECH_STACK.map((s) => (
            <Row key={s.name} title={<>{s.name} <Badge tone="accent">{s.version}</Badge></>} description={`${s.layer}: ${s.role}`} />
          ))}
        </div>
        <p className="note">Also built with: {TOOLING.join(', ')}.</p>
      </Section>

      <Section title="Legal disclaimer & fair use notice">
        <div className="card">
          <p className="note" style={{ marginTop: 0 }}>
            Ultimate Windows Desktop Settings is an independent workflow utility. It serves as an organizational panel and a shortcut layer
            for Windows settings and tools.
          </p>
          <p className="note">
            Trademark acknowledgement: all product names, programs, third-party titles and logos shown or opened by this app (including
            Microsoft Windows, Google Chrome and related developer tools) are trademarks of their owners. They are shown only so you can
            identify your software, under nominative fair use, and imply no affiliation, sponsorship or endorsement by those owners.
          </p>
          <p className="note" style={{ marginBottom: 0 }}>
            Limitation of liability: this software is provided "as is", without warranty of any kind. The developer is not responsible for
            closed programs, changed settings or permission prompts that result from actions you take in this app.
          </p>
        </div>
      </Section>
    </>
  );
}
