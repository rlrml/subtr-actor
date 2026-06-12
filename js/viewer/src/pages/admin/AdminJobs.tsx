import { useState, useCallback } from 'react';
import { Play, Clock, CheckCircle, XCircle, AlertCircle, RefreshCw, Settings, Zap, Shield, BarChart3, RotateCcw } from 'lucide-react';
import { DataTable, Column } from '@/components/admin/DataTable';
import {
  useJobsStatus,
  useJobsList,
  useJobsConfig,
  useTriggerJob,
  JobExecutionView,
  JobType,
  JobStatus,
} from '@/hooks/useAdminApi';
import { formatDistanceToNow, format } from 'date-fns';
import { toast } from 'sonner';

// Job type display config
const JOB_TYPE_CONFIG: Record<JobType, { label: string; icon: typeof Zap; color: string; description: string }> = {
  recompilation: {
    label: 'Recompilation',
    icon: RefreshCw,
    color: 'text-blue-400',
    description: 'Recompiles replays with outdated framework versions to the latest format',
  },
  cheat_detection: {
    label: 'Cheat Detection',
    icon: Shield,
    color: 'text-orange-400',
    description: 'Analyzes replays for suspicious behavior using the anticheat service',
  },
  stats_calculation: {
    label: 'Stats Calculation',
    icon: BarChart3,
    color: 'text-green-400',
    description: 'Calculates missing player and team statistics for replays',
  },
  replay_reprocessing: {
    label: 'Replay Reprocessing',
    icon: RotateCcw,
    color: 'text-purple-400',
    description: 'Retries processing of replays that previously failed (parse + compile)',
  },
};

// Status badge component
function StatusBadge({ status }: { status: JobStatus }) {
  const configs: Record<string, { icon: typeof RefreshCw; color: string; animate: boolean }> = {
    running: { icon: RefreshCw, color: 'bg-blue-500/20 text-blue-400', animate: true },
    completed: { icon: CheckCircle, color: 'bg-green-500/20 text-green-400', animate: false },
    failed: { icon: XCircle, color: 'bg-red-500/20 text-red-400', animate: false },
    pending: { icon: Clock, color: 'bg-yellow-500/20 text-yellow-400', animate: false },
    skipped: { icon: AlertCircle, color: 'bg-gray-500/20 text-gray-400', animate: false },
  };

  const config = configs[status] || { icon: AlertCircle, color: 'bg-gray-500/20 text-gray-400', animate: false };
  const Icon = config.icon;

  return (
    <span className={`inline-flex items-center gap-1.5 text-xs px-2 py-1 rounded-full ${config.color}`}>
      <Icon className={`w-3.5 h-3.5 ${config.animate ? 'animate-spin' : ''}`} />
      {status}
    </span>
  );
}

// Job type badge
function JobTypeBadge({ type }: { type: JobType }) {
  const config = JOB_TYPE_CONFIG[type] || { label: type, icon: Settings, color: 'text-gray-400' };
  const Icon = config.icon;

  return (
    <span className={`inline-flex items-center gap-1.5 text-sm ${config.color}`}>
      <Icon className="w-4 h-4" />
      {config.label}
    </span>
  );
}

// Format duration
function formatDuration(ms: number | null | undefined): string {
  if (ms === null || ms === undefined) return '-';
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`;
}

// Running jobs card
function RunningJobsCard({ jobs }: { jobs: JobExecutionView[] }) {
  if (jobs.length === 0) {
    return (
      <div className="bg-gray-900/50 border border-gray-800 rounded-xl p-6">
        <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
          <Play className="w-5 h-5 text-green-400" />
          Running Jobs
        </h3>
        <p className="text-gray-400 text-sm">No jobs currently running</p>
      </div>
    );
  }

  return (
    <div className="bg-gray-900/50 border border-gray-800 rounded-xl p-6">
      <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
        <Play className="w-5 h-5 text-green-400" />
        Running Jobs ({jobs.length})
      </h3>
      <div className="space-y-3">
        {jobs.map((job) => (
          <div key={job.id} className="bg-gray-800/50 rounded-lg p-4">
            <div className="flex items-center justify-between mb-2">
              <JobTypeBadge type={job.jobType} />
              <StatusBadge status={job.status} />
            </div>
            <div className="text-sm text-gray-400">
              Started {formatDistanceToNow(new Date(job.startedAt), { addSuffix: true })}
            </div>
            {job.totalItems !== null && (
              <div className="mt-2">
                <div className="flex justify-between text-xs text-gray-500 mb-1">
                  <span>Progress</span>
                  <span>{job.processedItems} / {job.totalItems}</span>
                </div>
                <div className="h-1.5 bg-gray-700 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-blue-500 transition-all"
                    style={{ width: `${(job.processedItems / job.totalItems) * 100}%` }}
                  />
                </div>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

// Queue stats card
function QueueStatsCard({ stats }: {
  stats: {
    replayProcessing: { waiting: number; active: number; completed: number; failed: number };
    cheatDetection: { waiting: number; active: number; completed: number; failed: number };
  };
}) {
  return (
    <div className="bg-gray-900/50 border border-gray-800 rounded-xl p-6">
      <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
        <BarChart3 className="w-5 h-5 text-violet-400" />
        Queue Statistics
      </h3>
      <div className="space-y-4">
        <div>
          <h4 className="text-sm font-medium text-gray-300 mb-2">Replay Processing</h4>
          <div className="grid grid-cols-4 gap-2">
            <div className="text-center">
              <div className="text-xl font-bold text-yellow-400">{stats.replayProcessing.waiting}</div>
              <div className="text-xs text-gray-500">Waiting</div>
            </div>
            <div className="text-center">
              <div className="text-xl font-bold text-blue-400">{stats.replayProcessing.active}</div>
              <div className="text-xs text-gray-500">Active</div>
            </div>
            <div className="text-center">
              <div className="text-xl font-bold text-green-400">{stats.replayProcessing.completed}</div>
              <div className="text-xs text-gray-500">Done</div>
            </div>
            <div className="text-center">
              <div className="text-xl font-bold text-red-400">{stats.replayProcessing.failed}</div>
              <div className="text-xs text-gray-500">Failed</div>
            </div>
          </div>
        </div>
        <div className="border-t border-gray-700 pt-4">
          <h4 className="text-sm font-medium text-gray-300 mb-2">Cheat Detection</h4>
          <div className="grid grid-cols-4 gap-2">
            <div className="text-center">
              <div className="text-xl font-bold text-yellow-400">{stats.cheatDetection.waiting}</div>
              <div className="text-xs text-gray-500">Waiting</div>
            </div>
            <div className="text-center">
              <div className="text-xl font-bold text-blue-400">{stats.cheatDetection.active}</div>
              <div className="text-xs text-gray-500">Active</div>
            </div>
            <div className="text-center">
              <div className="text-xl font-bold text-green-400">{stats.cheatDetection.completed}</div>
              <div className="text-xs text-gray-500">Done</div>
            </div>
            <div className="text-center">
              <div className="text-xl font-bold text-red-400">{stats.cheatDetection.failed}</div>
              <div className="text-xs text-gray-500">Failed</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// Last executions card
function LastExecutionsCard({
  executions,
  onTrigger,
  triggering,
}: {
  executions: Record<JobType, JobExecutionView | null>;
  onTrigger: (type: JobType) => void;
  triggering: boolean;
}) {
  const jobTypes: JobType[] = ['recompilation', 'cheat_detection', 'stats_calculation', 'replay_reprocessing'];

  return (
    <div className="bg-gray-900/50 border border-gray-800 rounded-xl p-6">
      <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
        <Clock className="w-5 h-5 text-blue-400" />
        Last Executions
      </h3>
      <div className="space-y-4">
        {jobTypes.map((type) => {
          const exec = executions[type];
          const config = JOB_TYPE_CONFIG[type];
          const Icon = config.icon;

          return (
            <div key={type} className="bg-gray-800/50 rounded-lg p-4">
              <div className="flex items-center justify-between mb-1">
                <span className={`inline-flex items-center gap-1.5 text-sm font-medium ${config.color}`}>
                  <Icon className="w-4 h-4" />
                  {config.label}
                </span>
                <button
                  onClick={() => onTrigger(type)}
                  disabled={triggering}
                  className="px-3 py-1 text-xs bg-violet-600 hover:bg-violet-700 text-white rounded-lg transition-colors disabled:opacity-50"
                >
                  <Zap className="w-3 h-3 inline mr-1" />
                  Trigger
                </button>
              </div>
              <p className="text-xs text-gray-500 mb-2">{config.description}</p>
              {exec ? (
                <div className="text-sm space-y-1">
                  <div className="flex items-center gap-2">
                    <StatusBadge status={exec.status} />
                    {(exec.completedAt || exec.startedAt) && (
                      <span className="text-gray-400">
                        {formatDistanceToNow(new Date(exec.completedAt || exec.startedAt), { addSuffix: true })}
                      </span>
                    )}
                  </div>
                  <div className="text-gray-500 text-xs">
                    Processed: {exec.processedItems ?? 0} | Failed: {exec.failedItems ?? 0} | Skipped: {exec.skippedItems ?? 0}
                    {exec.durationMs && ` | Duration: ${formatDuration(exec.durationMs)}`}
                  </div>
                </div>
              ) : (
                <p className="text-gray-500 text-sm">Never executed</p>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

// Config card
function ConfigCard({ config }: {
  config: {
    enabled: boolean;
    schedules: { recompilation: string; cheat_detection: string; stats_calculation: string; replay_reprocessing: string };
    batchSizes: { recompilation: number; cheat_detection: number; stats_calculation: number; replay_reprocessing: number };
    lockTtlSeconds: number;
  };
}) {
  return (
    <div className="bg-gray-900/50 border border-gray-800 rounded-xl p-6">
      <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
        <Settings className="w-5 h-5 text-gray-400" />
        Configuration
      </h3>
      <div className="space-y-4">
        <div className="flex items-center gap-2">
          <span className={`w-2 h-2 rounded-full ${config.enabled ? 'bg-green-400' : 'bg-red-400'}`} />
          <span className="text-gray-300">
            Scheduler {config.enabled ? 'Enabled' : 'Disabled'}
          </span>
        </div>
        <div className="space-y-2 text-sm">
          <div className="grid grid-cols-3 gap-2 text-gray-400">
            <span>Job Type</span>
            <span>Schedule (Cron)</span>
            <span>Batch Size</span>
          </div>
          <div className="grid grid-cols-3 gap-2">
            <span className="text-blue-400">Recompilation</span>
            <code className="text-gray-300 bg-gray-800 px-2 py-0.5 rounded text-xs">{config.schedules.recompilation}</code>
            <span className="text-gray-300">{config.batchSizes.recompilation}</span>
          </div>
          <div className="grid grid-cols-3 gap-2">
            <span className="text-orange-400">Cheat Detection</span>
            <code className="text-gray-300 bg-gray-800 px-2 py-0.5 rounded text-xs">{config.schedules.cheat_detection}</code>
            <span className="text-gray-300">{config.batchSizes.cheat_detection}</span>
          </div>
          <div className="grid grid-cols-3 gap-2">
            <span className="text-green-400">Stats Calculation</span>
            <code className="text-gray-300 bg-gray-800 px-2 py-0.5 rounded text-xs">{config.schedules.stats_calculation}</code>
            <span className="text-gray-300">{config.batchSizes.stats_calculation}</span>
          </div>
          <div className="grid grid-cols-3 gap-2">
            <span className="text-purple-400">Replay Reprocessing</span>
            <code className="text-gray-300 bg-gray-800 px-2 py-0.5 rounded text-xs">{config.schedules.replay_reprocessing}</code>
            <span className="text-gray-300">{config.batchSizes.replay_reprocessing}</span>
          </div>
        </div>
        <div className="text-xs text-gray-500 pt-2 border-t border-gray-700">
          Leader lock TTL: {config.lockTtlSeconds}s
        </div>
      </div>
    </div>
  );
}

export default function AdminJobs() {
  const [page, setPage] = useState(1);
  const [jobTypeFilter, setJobTypeFilter] = useState<JobType | ''>('');
  const [statusFilter, setStatusFilter] = useState<JobStatus | ''>('');

  // Fetch data with auto-refresh for status
  const { data: status, isLoading: statusLoading, refetch: refetchStatus } = useJobsStatus(5000);
  const { data: config, isLoading: configLoading } = useJobsConfig();
  const { data: jobs, pagination, isLoading: jobsLoading, refetch: refetchJobs } = useJobsList({
    page,
    limit: 20,
    jobType: jobTypeFilter || undefined,
    status: statusFilter || undefined,
  });

  const { triggerJob, isLoading: triggering } = useTriggerJob();

  const handleTrigger = useCallback(async (jobType: JobType) => {
    const result = await triggerJob(jobType);
    if (result) {
      toast.success(`${JOB_TYPE_CONFIG[jobType].label} job triggered`);
      refetchStatus();
      refetchJobs();
    } else {
      toast.error(`Failed to trigger ${JOB_TYPE_CONFIG[jobType].label} job`);
    }
  }, [triggerJob, refetchStatus, refetchJobs]);

  const columns: Column<JobExecutionView>[] = [
    {
      key: 'jobType',
      header: 'Job Type',
      render: (job) => <JobTypeBadge type={job.jobType} />,
    },
    {
      key: 'status',
      header: 'Status',
      render: (job) => <StatusBadge status={job.status} />,
    },
    {
      key: 'triggeredBy',
      header: 'Triggered By',
      render: (job) => (
        <span className={`text-sm ${job.triggeredBy === 'manual' ? 'text-violet-400' : 'text-gray-400'}`}>
          {job.triggeredBy === 'manual' ? 'Manual' : 'Scheduled'}
        </span>
      ),
    },
    {
      key: 'items',
      header: 'Items',
      render: (job) => (
        <div className="text-sm">
          <span className="text-green-400">{job.processedItems}</span>
          <span className="text-gray-500"> / </span>
          <span className="text-gray-300">{job.totalItems ?? '-'}</span>
          {job.failedItems > 0 && (
            <span className="text-red-400 ml-2">({job.failedItems} failed)</span>
          )}
        </div>
      ),
    },
    {
      key: 'duration',
      header: 'Duration',
      render: (job) => (
        <span className="text-gray-400 text-sm">{formatDuration(job.durationMs)}</span>
      ),
    },
    {
      key: 'startedAt',
      header: 'Started',
      render: (job) => (
        <span className="text-gray-400 text-sm" title={format(new Date(job.startedAt), 'PPpp')}>
          {formatDistanceToNow(new Date(job.startedAt), { addSuffix: true })}
        </span>
      ),
    },
    {
      key: 'error',
      header: '',
      render: (job) => job.errorMessage ? (
        <span title={job.errorMessage}>
          <AlertCircle className="w-4 h-4 text-red-400 cursor-help" />
        </span>
      ) : null,
    },
  ];

  const isLoading = statusLoading || configLoading;

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Scheduled Jobs</h1>
        <p className="text-gray-400 mt-1">Monitor and manage nightly maintenance jobs</p>
      </div>

      {/* Status Overview */}
      {!isLoading && status && config && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <RunningJobsCard jobs={status.running} />
          <QueueStatsCard stats={status.queueStats} />
          <LastExecutionsCard
            executions={status.lastExecutions}
            onTrigger={handleTrigger}
            triggering={triggering}
          />
          <ConfigCard config={config} />
        </div>
      )}

      {/* Job History */}
      <div className="bg-gray-900/50 border border-gray-800 rounded-xl p-6">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6">
          <h3 className="text-lg font-semibold text-white">Job History</h3>
          <div className="flex gap-2">
            <select
              value={jobTypeFilter}
              onChange={(e) => {
                setJobTypeFilter(e.target.value as JobType | '');
                setPage(1);
              }}
              className="px-3 py-1.5 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:border-violet-500"
            >
              <option value="">All types</option>
              <option value="recompilation">Recompilation</option>
              <option value="cheat_detection">Cheat Detection</option>
              <option value="stats_calculation">Stats Calculation</option>
              <option value="replay_reprocessing">Replay Reprocessing</option>
            </select>
            <select
              value={statusFilter}
              onChange={(e) => {
                setStatusFilter(e.target.value as JobStatus | '');
                setPage(1);
              }}
              className="px-3 py-1.5 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:border-violet-500"
            >
              <option value="">All statuses</option>
              <option value="running">Running</option>
              <option value="completed">Completed</option>
              <option value="failed">Failed</option>
            </select>
          </div>
        </div>

        <DataTable
          data={jobs || []}
          columns={columns}
          keyExtractor={(job) => job.id}
          pagination={pagination}
          onPageChange={setPage}
          loading={jobsLoading}
          emptyMessage="No job executions found"
        />
      </div>
    </div>
  );
}
