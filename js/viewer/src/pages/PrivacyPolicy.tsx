import { Link } from 'react-router-dom';
import { SEOHead } from '@/components/SEO';
import { Shield, ArrowLeft } from 'lucide-react';

export default function PrivacyPolicy() {
  return (
    <div className="py-12 px-4">
      <SEOHead
        title="Privacy Policy"
        description="Privacy Policy for BallCam - Learn how we collect, use, and protect your personal data."
      />

      <div className="max-w-3xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <Link
            to="/"
            className="inline-flex items-center gap-2 text-gray-400 hover:text-violet-400 transition-colors mb-6"
          >
            <ArrowLeft className="w-4 h-4" />
            Back to Home
          </Link>

          <div className="flex items-center gap-4 mb-4">
            <div className="inline-flex items-center justify-center w-12 h-12 rounded-full bg-violet-500/20">
              <Shield className="w-6 h-6 text-violet-400" />
            </div>
            <h1 className="text-3xl font-bold text-white">Privacy Policy</h1>
          </div>

          <p className="text-gray-400">
            Last updated: {new Date().toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' })}
          </p>
        </div>

        {/* Content */}
        <div className="space-y-8 text-gray-300">
          <section>
            <h2 className="text-xl font-semibold text-white mb-3">1. Introduction</h2>
            <p className="leading-relaxed mb-3">
              Welcome to BallCam. We respect your privacy and are committed to protecting your personal data
              in accordance with the General Data Protection Regulation (GDPR) and French data protection laws.
            </p>
            <p className="leading-relaxed">
              This privacy policy explains how we collect, use, and safeguard your information when you use our
              Rocket League replay viewing service. BallCam is operated from France and data is stored on
              self-hosted infrastructure located in France.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">2. Data We Collect</h2>
            <p className="leading-relaxed mb-3">We collect the following types of information:</p>
            <ul className="list-disc list-inside space-y-2 ml-4">
              <li>
                <strong className="text-white">Account Information:</strong> Username, email address, and password (encrypted)
                when you create an account.
              </li>
              <li>
                <strong className="text-white">Replay Data:</strong> Rocket League replay files you upload, including
                in-game player names, statistics, and match data.
              </li>
              <li>
                <strong className="text-white">Usage Data:</strong> Information about how you interact with our service,
                including pages visited, features used, and viewing history.
              </li>
              <li>
                <strong className="text-white">Technical Data:</strong> IP address, browser type, device information,
                and cookies for authentication and analytics.
              </li>
            </ul>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">3. How We Use Your Data</h2>
            <p className="leading-relaxed mb-3">We use your data to:</p>
            <ul className="list-disc list-inside space-y-2 ml-4">
              <li>Provide and maintain our replay viewing service</li>
              <li>Process and display your uploaded replays</li>
              <li>Enable features like comments, likes, and collaborative viewing</li>
              <li>Improve our service and develop new features</li>
              <li>Send important service updates and notifications</li>
              <li>Ensure security and prevent abuse</li>
            </ul>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">4. Cookies and Tracking</h2>
            <p className="leading-relaxed">
              We use essential cookies for authentication and session management. These cookies are necessary
              for the service to function properly. We may also use analytics cookies to understand how users
              interact with our service, helping us improve the user experience.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">5. Data Sharing and Transfers</h2>
            <p className="leading-relaxed mb-3">
              We do not sell your personal data. We may share data in the following circumstances:
            </p>
            <ul className="list-disc list-inside space-y-2 ml-4">
              <li>
                <strong className="text-white">Public Content:</strong> Replays you upload are publicly viewable
                by default, including player names and statistics from the replay.
              </li>
              <li>
                <strong className="text-white">Service Providers:</strong> We may use third-party services for
                analytics that process data on our behalf, with appropriate safeguards.
              </li>
              <li>
                <strong className="text-white">Legal Requirements:</strong> We may disclose data if required by
                French or EU law, or to protect our rights and safety.
              </li>
            </ul>
            <p className="leading-relaxed mt-3">
              <strong className="text-white">Data Location:</strong> Your data is stored on servers located in
              France and is not transferred outside the European Economic Area (EEA).
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">6. Data Security</h2>
            <p className="leading-relaxed">
              We implement appropriate security measures to protect your data, including encryption of passwords,
              secure HTTPS connections, and regular security audits. However, no method of transmission over the
              Internet is 100% secure, and we cannot guarantee absolute security.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">7. Data Retention</h2>
            <p className="leading-relaxed">
              We retain your account data as long as your account is active. Replay data is stored indefinitely
              unless you delete it or request account deletion. You can delete your uploaded replays at any time
              through your account settings.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">8. Your Rights (GDPR)</h2>
            <p className="leading-relaxed mb-3">
              Under the General Data Protection Regulation (GDPR), you have the following rights:
            </p>
            <ul className="list-disc list-inside space-y-2 ml-4">
              <li>
                <strong className="text-white">Right of access:</strong> Obtain confirmation of whether your data
                is being processed and receive a copy of it.
              </li>
              <li>
                <strong className="text-white">Right to rectification:</strong> Request correction of inaccurate
                or incomplete personal data.
              </li>
              <li>
                <strong className="text-white">Right to erasure:</strong> Request deletion of your personal data
                ("right to be forgotten").
              </li>
              <li>
                <strong className="text-white">Right to restriction:</strong> Request limitation of processing
                of your personal data.
              </li>
              <li>
                <strong className="text-white">Right to data portability:</strong> Receive your data in a
                structured, commonly used format.
              </li>
              <li>
                <strong className="text-white">Right to object:</strong> Object to processing of your personal
                data for specific purposes.
              </li>
            </ul>
            <p className="leading-relaxed mt-3">
              To exercise these rights, please{' '}
              <Link to="/contact" className="text-violet-400 hover:text-violet-300 underline">
                contact us
              </Link>
              . We will respond within 30 days as required by GDPR. You also have the right to lodge a complaint
              with the French data protection authority (CNIL) at{' '}
              <a
                href="https://www.cnil.fr"
                target="_blank"
                rel="noopener noreferrer"
                className="text-violet-400 hover:text-violet-300 underline"
              >
                www.cnil.fr
              </a>
              .
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">9. Children's Privacy</h2>
            <p className="leading-relaxed">
              Our service is not intended for children under 13 years of age. We do not knowingly collect
              personal data from children. If you believe a child has provided us with personal data,
              please contact us so we can take appropriate action.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">10. Changes to This Policy</h2>
            <p className="leading-relaxed">
              We may update this privacy policy from time to time. We will notify you of any significant
              changes by posting the new policy on this page and updating the "Last updated" date.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">11. Contact Us</h2>
            <p className="leading-relaxed">
              If you have any questions about this privacy policy or our data practices, please{' '}
              <Link to="/contact" className="text-violet-400 hover:text-violet-300 underline">
                contact us
              </Link>
              .
            </p>
          </section>
        </div>
      </div>
    </div>
  );
}
