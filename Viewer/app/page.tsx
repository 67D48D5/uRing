'use client';

import { useEffect, useState } from 'react';
import {
  fetchAllNotices,
  fetchNoticesByCampus,
  type Notice,
  type ApiResponse,
} from '@/lib/api';

export default function Home() {
  const [notices, setNotices] = useState<Notice[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedCampus, setSelectedCampus] = useState<string | null>(null);
  const [selectedDept, setSelectedDept] = useState<string | null>(null);

  // Get unique campuses and departments
  const campuses = Array.from(
    new Set(notices.map((notice) => notice.campus))
  ).sort();

  const departments = selectedCampus
    ? Array.from(
      new Set(
        notices
          .filter((notice) => notice.campus === selectedCampus)
          .map((notice) => notice.department_name)
      )
    ).sort()
    : [];

  const filteredNotices = notices.filter((notice) => {
    if (selectedCampus && notice.campus !== selectedCampus) return false;
    if (selectedDept && notice.department_name !== selectedDept) return false;
    return true;
  });

  useEffect(() => {
    const loadData = async () => {
      setLoading(true);
      setError(null);

      const response: ApiResponse = await fetchAllNotices();

      if (response.status === 'success') {
        setNotices(response.data);
      } else {
        setError(response.message || 'Failed to load notices');
      }

      setLoading(false);
    };

    loadData();
  }, []);

  const handleCampusChange = (campus: string | null) => {
    setSelectedCampus(campus);
    setSelectedDept(null);
  };

  const handleDeptChange = (dept: string | null) => {
    setSelectedDept(dept);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <p className="text-lg text-gray-500">Loading notices...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <p className="text-lg text-red-500">{error}</p>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 py-8">
        <h1 className="text-3xl font-bold text-gray-900 mb-8">
          Yonsei Campus Notices
        </h1>

        {/* Filters */}
        <div className="bg-white rounded-lg shadow-md p-6 mb-8">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* Campus Filter */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-3">
                Campus
              </label>
              <div className="space-y-2">
                <button
                  onClick={() => handleCampusChange(null)}
                  className={`block w-full text-left px-4 py-2 rounded-md transition ${selectedCampus === null
                    ? 'bg-blue-500 text-white'
                    : 'bg-gray-100 hover:bg-gray-200 text-gray-700'
                    }`}
                >
                  All Campuses
                </button>
                {campuses.map((campus) => (
                  <button
                    key={campus}
                    onClick={() => handleCampusChange(campus)}
                    className={`block w-full text-left px-4 py-2 rounded-md transition ${selectedCampus === campus
                      ? 'bg-blue-500 text-white'
                      : 'bg-gray-100 hover:bg-gray-200 text-gray-700'
                      }`}
                  >
                    {campus}
                  </button>
                ))}
              </div>
            </div>

            {/* Department Filter */}
            {selectedCampus && (
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-3">
                  Department
                </label>
                <div className="space-y-2">
                  <button
                    onClick={() => handleDeptChange(null)}
                    className={`block w-full text-left px-4 py-2 rounded-md transition ${selectedDept === null
                      ? 'bg-blue-500 text-white'
                      : 'bg-gray-100 hover:bg-gray-200 text-gray-700'
                      }`}
                  >
                    All Departments
                  </button>
                  {departments.map((dept) => (
                    <button
                      key={dept}
                      onClick={() => handleDeptChange(dept)}
                      className={`block w-full text-left px-4 py-2 rounded-md transition ${selectedDept === dept
                        ? 'bg-blue-500 text-white'
                        : 'bg-gray-100 hover:bg-gray-200 text-gray-700'
                        }`}
                    >
                      {dept}
                    </button>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Notice Count */}
        <p className="text-sm text-gray-600 mb-4">
          Showing {filteredNotices.length} of {notices.length} notices
        </p>

        {/* Notices List */}
        <div className="space-y-4">
          {filteredNotices.length === 0 ? (
            <div className="bg-white rounded-lg shadow-md p-8 text-center">
              <p className="text-gray-500">No notices found</p>
            </div>
          ) : (
            filteredNotices.map((notice, index) => (
              <a
                key={index}
                href={notice.link}
                target="_blank"
                rel="noopener noreferrer"
                className="block bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition"
              >
                <div className="flex justify-between items-start mb-3">
                  <div className="flex-1">
                    <h3 className="text-lg font-semibold text-gray-900 hover:text-blue-600 transition">
                      {notice.title}
                    </h3>
                  </div>
                  <div className="ml-4 text-right">
                    <p className="text-sm text-gray-500">{notice.date}</p>
                  </div>
                </div>

                <div className="flex flex-wrap gap-2 mb-2">
                  <span className="inline-block bg-blue-100 text-blue-700 px-3 py-1 rounded-full text-xs font-medium">
                    {notice.campus}
                  </span>
                  <span className="inline-block bg-green-100 text-green-700 px-3 py-1 rounded-full text-xs font-medium">
                    {notice.department_name}
                  </span>
                  <span className="inline-block bg-gray-100 text-gray-700 px-3 py-1 rounded-full text-xs font-medium">
                    {notice.board_name}
                  </span>
                </div>

                <p className="text-xs text-gray-400">
                  {notice.board_id} • {notice.department_id}
                </p>
              </a>
            ))
          )}
        </div>
      </div>
    </div>
  );
}