import { Link } from 'react-router-dom';
import { SEOHead } from '@/components/SEO';
import { Scale, ArrowLeft } from 'lucide-react';

export default function LegalNotice() {
  return (
    <div className="py-12 px-4">
      <SEOHead
        title="Legal Notice"
        description="Legal Notice and Terms of Service for BallCam - Rocket League replay viewer."
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
              <Scale className="w-6 h-6 text-violet-400" />
            </div>
            <h1 className="text-3xl font-bold text-white">Legal Notice</h1>
          </div>

          <p className="text-gray-400">
            Last updated: {new Date().toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' })}
          </p>
        </div>

        {/* Content */}
        <div className="space-y-8 text-gray-300">
          <section>
            <h2 className="text-xl font-semibold text-white mb-3">1. Service Provider</h2>
            <p className="leading-relaxed mb-3">
              In accordance with Article 6 of French Law No. 2004-575 of June 21, 2004 (LCEN), the following
              information is provided:
            </p>
            <p className="leading-relaxed mb-3">
              <strong className="text-white">Publisher:</strong> BallCam is a personal, non-commercial project
              developed and maintained by an individual based in France. This service is provided free of charge
              for the Rocket League community.
            </p>
            <p className="leading-relaxed mb-3">
              <strong className="text-white">Hosting:</strong> This website is self-hosted on a personal Kubernetes
              cluster located in France.
            </p>
            <p className="leading-relaxed">
              <strong className="text-white">Contact:</strong> For any inquiries, please use our{' '}
              <Link to="/contact" className="text-violet-400 hover:text-violet-300 underline">
                contact form
              </Link>
              .
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">2. Terms of Service</h2>
            <p className="leading-relaxed mb-3">
              By accessing and using BallCam, you agree to the following terms:
            </p>
            <ul className="list-disc list-inside space-y-2 ml-4">
              <li>You must be at least 13 years old to use this service.</li>
              <li>You are responsible for maintaining the security of your account.</li>
              <li>You agree not to misuse the service or help anyone else do so.</li>
              <li>You will not upload content that violates any laws or third-party rights.</li>
              <li>We reserve the right to suspend or terminate accounts that violate these terms.</li>
            </ul>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">3. Intellectual Property</h2>
            <p className="leading-relaxed mb-3">
              <strong className="text-white">Rocket League:</strong> Rocket League is a trademark of Psyonix LLC.
              BallCam is not affiliated with, endorsed, sponsored, or specifically approved by Psyonix LLC or Epic Games, Inc.
            </p>
            <p className="leading-relaxed mb-3">
              <strong className="text-white">User Content:</strong> You retain ownership of the replay files you upload.
              By uploading content, you grant BallCam a non-exclusive license to store, process, and display your replays
              on the platform.
            </p>
            <p className="leading-relaxed">
              <strong className="text-white">Service Content:</strong> The BallCam website design, code, and original
              content are the property of the developer. Third-party 3D models and assets are used with permission
              and credited accordingly.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">4. Acceptable Use</h2>
            <p className="leading-relaxed mb-3">You agree not to:</p>
            <ul className="list-disc list-inside space-y-2 ml-4">
              <li>Upload malicious files or attempt to exploit the service</li>
              <li>Harass, abuse, or harm other users through comments or other features</li>
              <li>Impersonate other users or misrepresent your identity</li>
              <li>Use automated systems to scrape data or overload the service</li>
              <li>Attempt to gain unauthorized access to accounts or systems</li>
              <li>Upload content that infringes on intellectual property rights</li>
            </ul>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">5. User Generated Content</h2>
            <p className="leading-relaxed mb-3">
              Users may upload replay files and post comments. You are solely responsible for the content you submit.
            </p>
            <p className="leading-relaxed">
              We reserve the right to remove any content that violates these terms or is deemed inappropriate,
              without prior notice. Repeated violations may result in account suspension or termination.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">6. Disclaimer of Warranties</h2>
            <p className="leading-relaxed">
              BallCam is provided "as is" and "as available" without warranties of any kind, either express or implied.
              We do not guarantee that the service will be uninterrupted, secure, or error-free. We are not responsible
              for any loss of data or damage resulting from your use of the service.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">7. Limitation of Liability</h2>
            <p className="leading-relaxed">
              To the maximum extent permitted by law, BallCam and its developer shall not be liable for any indirect,
              incidental, special, consequential, or punitive damages, or any loss of profits or revenues, whether
              incurred directly or indirectly, or any loss of data, use, goodwill, or other intangible losses.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">8. Service Modifications</h2>
            <p className="leading-relaxed">
              We reserve the right to modify, suspend, or discontinue the service (or any part thereof) at any time,
              with or without notice. We shall not be liable to you or any third party for any modification,
              suspension, or discontinuation of the service.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">9. Account Termination</h2>
            <p className="leading-relaxed">
              You may delete your account at any time. We may terminate or suspend your account immediately,
              without prior notice, for conduct that we believe violates these terms or is harmful to other users,
              us, or third parties, or for any other reason at our sole discretion.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">10. Changes to Terms</h2>
            <p className="leading-relaxed">
              We may revise these terms from time to time. The most current version will always be posted on this page.
              By continuing to use BallCam after revisions become effective, you agree to be bound by the revised terms.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">11. Governing Law</h2>
            <p className="leading-relaxed">
              These terms are governed by French law. Any dispute arising from the use of this service shall
              be subject to the exclusive jurisdiction of the French courts.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">12. Future Changes</h2>
            <p className="leading-relaxed">
              BallCam is currently a free, non-commercial service. In the future, we may introduce paid plans
              or additional features. If such changes occur, these terms will be updated accordingly, and users
              will be notified in advance.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-3">13. Contact</h2>
            <p className="leading-relaxed">
              For any questions regarding these terms or the service, please{' '}
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
